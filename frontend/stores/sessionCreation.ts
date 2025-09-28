import type { DateValue } from "@internationalized/date"
import type { Row } from "@tanstack/vue-table";
import Stations from "~/pages/sessions/new/stations.vue";

export const useSessionCreationStore = defineStore("sessionCreation", {
  state: () => ({
    // interval storage = microseconds, interval display = seconds
    payload: {
        session: { 
            scheduled_date: null,
            location: "",
            intermission_duration: { months: 0, days: 0, microseconds: 0 },
            feedback: false,
            feedback_duration: null,
            static_at_end: false,
        } as SessionPayload,
        stations: [] as StationPayload[],
        slots: [] as SlotPayload[],
    },
    templates: [] as TemplateSessionWithStations[],
    isDirty: false, // track unsaved changes
    fetchedTemplate: false,
  }),
  actions: {
    setDirty(dirty = true) {
      this.isDirty = dirty;
    },
    resetpayload() {
      this.payload = {
        session: {
            scheduled_date: null,
            location: "",
            intermission_duration: { months: 0, days: 0, microseconds: 0 },
            feedback: false,
            feedback_duration: null,
            static_at_end: false,
        } as SessionPayload,
        stations: [] as StationPayload[],
        slots: [] as SlotPayload[],
      };
      this.isDirty = false;
    },
    async fetchTemplates() {
        try {
            const response = await apiFetch("/templates/get-all", {
                method: "GET",
            });
            this.templates = response;
        } catch (err) {
            toast.error("Failed to get templates: ", err.data);
        } finally {
            this.fetchedTemplate = true;
        }
    },
    // updateIntermissionDuration(seconds: number) {
    //     this.payload.session.intermission_duration = {
    //         months: 0,
    //         days: 0,
    //         microseconds: seconds * 1_000_000,
    //     };
    //     this.setDirty();
    // },
    // updateFeedbackDuration(seconds: number) {
    //     this.payload.session.feedback_duration = {
    //         months: 0,
    //         days: 0,
    //         microseconds: seconds * 1_000_000,
    //     };
    //     this.setDirty();
    // },
    // updateStationDuration(index: number, seconds: number) {
    //     this.payload.stations[index].duration = {
    //         months: 0,
    //         days: 0,
    //         microseconds: seconds * 1_000_000,
    //     };
    //     this.setDirty();
    // },
    addStation(title = "", seconds = 60) {
        const nextIndex = this.payload.stations.length;
        this.payload.stations = [...this.payload.stations,
            {
                title,
                index: nextIndex,
                duration: { months: 0, days: 0, microseconds: seconds * 1_000_000 },
        }];
        this.setDirty();
    },
    updateStation(rowIndex: number, columnId: string, value: any) {
        if (this.payload.stations[rowIndex]) {
            this.payload.stations[rowIndex][columnId] = value;
        }
    },
    deleteSelectedStations(selected: Row<IStationPayload>[]) {
        const indexes = selected.map(row => row.original.index);
        this.payload.stations = this.payload.stations.filter(station => !indexes.includes(station.index));
        this.payload.stations.forEach((station, idx) => station.index = idx);
        this.setDirty();
    },
    reorderStation(rowIndex: number, targetRowIndex: number, instruction: string) {
        if (rowIndex === targetRowIndex) return;
        if (this.payload.stations.length <= rowIndex || this.payload.stations.length <= targetRowIndex) return;
        if (instruction === "reorder-before") {
            if (rowIndex > targetRowIndex) {
                this.stationToOrder(rowIndex, targetRowIndex)
            } else {
                if (rowIndex == targetRowIndex - 1) {
                    return;
                }
                this.stationToOrder(rowIndex, targetRowIndex - 1)
            }
        }
        if (instruction === "reorder-after") {
            if (rowIndex > targetRowIndex) {
                if (rowIndex == targetRowIndex + 1) {
                    return;
                }
                this.stationToOrder(rowIndex, targetRowIndex + 1)
            } else {
                this.stationToOrder(rowIndex, targetRowIndex)
            }
        }
    },
    stationToOrder(rowIndex: number, newIndex: number) {
        let newStations = this.payload.stations.toSpliced(rowIndex, 1);
        newStations.splice(newIndex, 0, this.payload.stations[rowIndex])
        newStations.forEach((station, index) => station.index = index);
        console.log("Triggered");
        console.log(newStations);
        console.log(this.payload.stations);
        this.payload.stations = newStations
    },
    addCircuit(slot_index: number) {
        const slot = this.payload.slots[slot_index];
        if (slot) {
            const new_circuit = { female_only: false } as CircuitPayload;
            slot.circuits.push(new_circuit);
            this.setDirty();
        }
    },
    addSlot() {
        const slot_index = this.payload.slots.length;
        const new_slot = { runs: [], circuits: [] } as SlotPayload;
        this.payload.slots.push(new_slot);
        this.setDirty();
        this.addRun(slot_index);
        return slot_index;
    },
    removeSlot(slot_index: number) {
        if (slot_index > -1) {
            this.payload.slots.splice(slot_index, 1);
            this.setDirty();
        }
    },
    addRun(slot_index: number) {
        const slot = this.payload.slots[slot_index];
        if (slot) {
            const slot_runs = slot.runs.length;
            let last_time = "08:00";
            if (slot_runs > 0) {
                last_time = slot.runs[slot_runs-1].scheduled_end;
            }
            let [hours, minutes] = last_time.split(":").map(Number);
            let run_duration = this.calculateRunDuration() / 1_000_000 / 60;
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
    calculateRunDuration() { // run duration in microseconds
        let run_duration: number = this.payload.session.intermission_duration.microseconds * this.payload.stations.length;
        console.log("Total Intermission Duration (seconds): ", run_duration / 1_000_000);
        if (this.payload.session.feedback && this.payload.session.feedback_duration) {
            run_duration += this.payload.session.feedback_duration.microseconds * this.payload.stations.length;
        }
        console.log("Total Intermission + Feedback Duration (seconds): ", run_duration / 1_000_000);
        if (this.payload.session.static_at_end) {
            run_duration += this.payload.stations[0].duration.microseconds * (this.payload.stations.length - 1);
            run_duration += this.payload.stations[this.payload.stations.length-1].duration.microseconds;
        } else {
            run_duration += this.payload.stations[0].duration.microseconds * this.payload.stations.length;
        }
        console.log("Total Run Duration (seconds): ", run_duration / 1_000_000);
        return run_duration;
    },
    updateScheduledEnd(start_time: string, slot_index: number, run_index: number) { // HH:MM
        if (start_time.includes("_") || start_time == "") {
            return;
        }
        let [hours, minutes] = start_time.split(":").map(Number);
        const run_duration = this.calculateRunDuration() / 1_000_000 / 60;
        let total_minutes = hours * 60 + minutes + run_duration;
        total_minutes = total_minutes % (24 * 60);
        if (total_minutes < 0) total_minutes += 24 * 60;
        const new_hours = Math.floor(total_minutes / 60);
        const new_minutes = total_minutes % 60;
        const new_time = new_hours.toString().padStart(2, "0") + ":" + new_minutes.toString().padStart(2, "0");
        const slot = this.payload.slots[slot_index];
        if (slot) {
            slot.runs[run_index].scheduled_end = new_time;
            this.setDirty();
        }
    },
    applyTemplate(template: TemplateSessionWithStations) {
        this.payload = {
            ...this.payload,
            session: {
                ...this.payload.session,
                intermission_duration: template.intermission_duration,
                feedback: template.feedback,
                feedback_duration: template.feedback_duration,
                static_at_end: template.static_at_end,
            },
            stations: template.stations.map(({ id, template_id, ...station }) => station),
        };
        this.setDirty();
        },
    },
});