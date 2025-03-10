import { defineStore } from "pinia";
import Slots from "~/pages/sessions/new/slots.vue";
import { apiFetch } from "~/composables/apiFetch"

export const useSessionCreationStore = defineStore("sessionCreation", {
  state: () => ({
    form: {
      session: {
        organisation: "",
        scheduled_date: null,
        location: "",
        intermission_duration: { months: 0, days: 0, microseconds: 0 },
        feedback: false,
        feedback_duration: { months: 0, days: 0, microseconds: 0 },
        static_at_end: false,
      },
      stations: [] as StationPayload[],
      slots: [] as SlotPayload[],
    },
    intermissionSeconds: 0,
    feedbackSeconds: 0,
    stationsMinutes: [] as StationMinutePayload[],
    step: 1,
    isDirty: false, // Track unsaved changes
  }),
  actions: {
    setDirty(dirty = true) {
      this.isDirty = dirty;
    },
    resetForm() {
      this.form = {
        session: {
          organisation: "",
          scheduled_date: null,
          location: "",
          intermission_duration: { months: 0, days: 0, microseconds: 0 },
          feedback: false,
          feedback_duration: { months: 0, days: 0, microseconds: 0 },
          static_at_end: false,
        },
        stations: [] as StationPayload[],
        slots: [] as SlotPayload[],
      };
      this.intermissionSeconds = 0;
      this.feedbackSeconds = 0;
      this.stationsMinutes = [] as StationMinutePayload[];
      this.step = 1;
      this.isDirty = false;
    },
    updateIntermissionDuration() {
      this.form.session.intermission_duration = {
        months: 0,
        days: 0,
        microseconds: this.intermissionSeconds * 1_000_000,
      };
    },
    updateFeedbackDuration() {
      this.form.session.feedback_duration = {
        months: 0,
        days: 0,
        microseconds: this.feedbackSeconds * 1_000_000,
      };
    },
    updateStationDuration(index) {
      this.form.stations[index].duration = {
        months: 0,
        days: 0,
        microseconds: this.stationsMinutes[index].duration * 60 * 1_000_000,
      }
      this.setDirty();
    },
    addStation(title = "", minutes = 1) {
      const nextIndex = this.form.stations.length;
      this.form.stations.push({
        title,
        index: nextIndex,
        duration: { months: 0, days: 0, microseconds: minutes * 60 * 1_000_000 },
      });
      this.stationsMinutes.push({
        title,
        index: nextIndex,
        duration: { months: 0, days: 0, microseconds: minutes },
      });
      this.setDirty();
    },
    removeStations(selected_stations) {
      if (selected_stations.length == this.form.stations.length) {
        this.form.stations = [] as StationPayload[];
        this.stationsMinutes = [] as StationMinutePayload[];
        return;
      }
      const newStations = [...this.form.stations];
      const newStationsMinutes = [...this.stationsMinutes];
      for (let i = 0; i < selected_stations.length; i++) {
        const index = newStations.indexOf(selected_stations[i]);
        if (index > -1) {
          newStations.splice(index, 1);
          newStationsMinutes.splice(index, 1);
        }
      }
      newStations.forEach((station, index) => station.index = index);
      newStationsMinutes.forEach((station, index) => station.index = index);
      this.form.stations = newStations;
      this.stationsMinutes = newStationsMinutes;
    },
    onStationRowReorder(event) {
      const newStations = [...this.form.stations];
      newStations.splice(event.dragIndex, 1);
      newStations.splice(event.dropIndex, 0, this.form.stations[event.dragIndex]);
      newStations.forEach((station, index) => station.index = index);
      const newStationsMinutes = [...this.stationsMinutes];
      newStationsMinutes.splice(event.dragIndex, 1)
      newStationsMinutes.splice(event.dropIndex, 0, this.stationsMinutes[event.dragIndex]);
      newStationsMinutes.forEach((station, index) => station.index = index);
      this.form.stations = newStations;
      this.stationsMinutes = newStationsMinutes;
      this.setDirty();
    },
    addCircuit(slot_key: string,) {
      const slot = this.form.slots.find(s => s.key === slot_key);
      if (slot) {
        const circuit_index = slot.circuits.length;
        const nextKey = String.fromCharCode(65 + circuit_index); // A, B, C, ...
        const new_circuit = { key: nextKey, female_only: false, index: circuit_index } as CircuitPayload;
        slot.circuits.push(new_circuit);
        this.setDirty();
      }
    },
    removeCircuits(slot_key: string, selected_circuits) {
      const slot = this.form.slots.find(s => s.key === slot_key);
      if (slot) {
        if (selected_circuits.length == slot.circuits.length) {
          slot.circuits = [] as CircuitPayload[];
          return;
        }
        const newCircuits = [...slot.circuits];
        for (let i = 0; i < selected_circuits.length; i++) {
          const index = newCircuits.indexOf(selected_circuits[i]);
          if (index > -1) {
            newCircuits.splice(index, 1);
          }
        }
        newCircuits.forEach((circuit, index) => circuit.index = index);
        slot.circuits = newCircuits;
      }
    },
    addSlot() {
      const slot_index = this.form.slots.length;
      const nextKey = String.fromCharCode(65 + slot_index); // A, B, C, ...
      const new_slot = { key: nextKey, runs: [], circuits: [] } as SlotPayload;
      this.form.slots.push(new_slot);
      this.setDirty();
      this.addRun(nextKey);
      return nextKey;
    },
    removeSlot(key: string) {
      const slot_index = this.form.slots.findIndex(s => s.key === key);
      if (slot_index > -1) {
        this.form.slots.splice(slot_index, 1);
        this.setDirty();
      }
    },
    addRun(key: string) {
      const slot = this.form.slots.find(s => s.key === key);
      if (slot) {
        const slot_runs = slot.runs.length;
        let last_time = "09:00";
        if (slot_runs > 0) {
          last_time = slot.runs[slot_runs-1].scheduled_end;
        }
        let [hours, minutes] = last_time.split(":").map(Number);
        let run_duration = this.calculateSlotDuration();
        let total_minutes = hours * 60 + minutes + run_duration;
        total_minutes = total_minutes % (24 * 60);
        if (total_minutes < 0) total_minutes += 24 * 60;
        const new_hours = Math.floor(total_minutes / 60);
        const new_minutes = total_minutes % 60;
        const new_time = new_hours.toString().padStart(2, "0") + ":" + new_minutes.toString().padStart(2, "0");
        slot.runs.push({ scheduled_start: last_time, scheduled_end: new_time, flip_allocation: false });
        this.setDirty();
      }
    },
    calculateSlotDuration() {
      let run_duration: number = (this.intermissionSeconds / 60) * this.form.stations.length;
      console.log("Total Intermission Duration: ", run_duration);
      if (this.form.session.feedback) {
        run_duration += (this.feedbackSeconds / 60) * this.form.stations.length;
      }
      console.log("Total Intermission + Feedback Duration: ", run_duration);
      if (this.form.session.static_at_end) {
        run_duration += this.stationsMinutes[0].duration * (this.form.stations.length - 1);
        run_duration += this.stationsMinutes[this.stationsMinutes.length-1].duration;
      } else {
        run_duration += this.stationsMinutes[0].duration * this.form.stations.length;
      }
      console.log("Total Slot Duration: ", run_duration);
      return run_duration;
    },
    updateScheduledEnd(start_time: string, key: string, index: number) { // HH:MM
      if (start_time.includes("_") || start_time == "") {
        return;
      }
      let [hours, minutes] = start_time.split(":").map(Number);
      const run_duration = this.calculateSlotDuration();
      let total_minutes = hours * 60 + minutes + run_duration;
      total_minutes = total_minutes % (24 * 60);
      if (total_minutes < 0) total_minutes += 24 * 60;
      const new_hours = Math.floor(total_minutes / 60);
      const new_minutes = total_minutes % 60;
      const new_time = new_hours.toString().padStart(2, "0") + ":" + new_minutes.toString().padStart(2, "0");
      const slot = this.form.slots.find(s => s.key === key);
      if (slot) {
        slot.runs[index].scheduled_end = new_time;
        this.setDirty();
      }
    },
    removeRuns(key: string, selected_runs) {
      const slot_index = this.form.slots.findIndex(s => s.key === key);
      if (slot_index > -1) {
        const slot_runs = this.form.slots[slot_index].runs;
        if (slot_runs.length > 0) {
          const newRuns = [...slot_runs];
          for (let i = 0; i < selected_runs.length; i++) {
            const index = newRuns.indexOf(selected_runs[i]);
            if (index > -1) {
              newRuns.splice(index, 1);
            }
          }
          newRuns.forEach((run, index) => run.index = index);
          this.form.slots[slot_index].runs = newRuns;
        }
        this.setDirty();
      }
    },
    applyTemplate(template) {
      this.form = {
        ...this.form,
        session: {
          ...this.form.session,
          intermission_duration: template.intermission_duration,
          feedback: template.feedback,
          feedback_duration: template.feedback_duration,
          static_at_end: template.static_at_end,
        },
        stations: template.stations.map(({ id, template_id, ...station }) => station),
      };
      this.intermissionSeconds = this.form.session.intermission_duration.microseconds / 1_000_000 ;
      this.feedbackSeconds = this.form.session.feedback_duration.microseconds / 1_000_000;
      this.stationsMinutes = template.stations.map(({ id, template_id, duration, ...station }) => {
        const newStation = {
          ...station,
          duration: duration.microseconds / 60 / 1_000_000,
        };
        return newStation;
      });
      console.log(this.stationsMinutes);
      this.setDirty();
    },
  },
});

interface StationPayload {
  title: string;
  index: number;
  duration: { months: 0, days: 0, microseconds: number };
}

interface StationMinutePayload {
  title: string;
  index: number;
  duration: number;
}

interface CircuitPayload {
  key: string;
  female_only: boolean;
}

interface RunPayload {
  flip_allocation: boolean;
  scheduled_start: string;
  scheduled_end: string;
}

interface SlotPayload {
  key: string;
  runs: RunPayload[];
  circuits: CircuitPayload[];
}