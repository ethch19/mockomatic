import { defineStore } from 'pinia';

export const useSessionCreationStore = defineStore('sessionCreation', {
  state: () => ({
    form: {
      session: {
        organisation: '',
        scheduled_date: null,
        location: '',
        intermission_duration: { months: 0, days: 0, microseconds: 0 },
        static_at_end: false,
      },
      stations: [] as StationPayload[],
      slots: [
        { slot_time: 'AM', runs: [], circuits: [] as CircuitPayload[] },
        { slot_time: 'PM', runs: [], circuits: [] as CircuitPayload[] },
      ],
    },
    intermissionMinutes: 0,
    intermissionSeconds: 0,
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
          organisation: '',
          scheduled_date: null,
          location: '',
          intermission_duration: { months: 0, days: 0, microseconds: 0 },
          static_at_end: false,
        },
        stations: [],
        slots: [
          { slot_time: 'AM', runs: [], circuits: [] },
          { slot_time: 'PM', runs: [], circuits: [] },
        ],
      };
      this.intermissionMinutes = 0;
      this.intermissionSeconds = 0;
      this.step = 1;
      this.isDirty = false;
    },
    updateIntermissionDuration() {
      const totalMicroseconds = (this.intermissionMinutes * 60 + this.intermissionSeconds) * 1_000_000;
      this.form.session.intermission_duration = {
        months: 0,
        days: 0,
        microseconds: totalMicroseconds,
      };
    },
    addStation(title = '', minutes = 1) {
      const nextIndex = this.form.stations.length;
      this.form.stations.push({
        title,
        index: nextIndex,
        duration: { months: 0, days: 0, microseconds: minutes * 60 * 1_000_000 },
        durationMinutes: minutes,
      });
      this.setDirty();
    },
    addCircuit() {
      const nextKey = String.fromCharCode(65 + this.form.slots[0].circuits.length); // A, B, C, ...
      this.form.slots[0].circuits.push({ key: nextKey, female_only: false });
      this.form.slots[1].circuits.push({ ...this.form.slots[0].circuits[this.form.slots[0].circuits.length - 1] });
      this.setDirty();
    },
    updateStationDuration(index, minutes) {
      this.form.stations[index].duration = {
        months: 0,
        days: 0,
        microseconds: minutes * 60 * 1_000_000,
      };
      this.form.stations[index].durationMinutes = minutes;
      this.setDirty();
    },
    onRowReorder(event) {
      const newStations = [...this.form.stations];
      newStations.splice(event.dragIndex, 1);
      newStations.splice(event.dropIndex, 0, this.form.stations[event.dragIndex]);
      newStations.forEach((station, index) => station.index = index);
      this.form.stations = newStations;
      this.setDirty();
    },
    applyTemplate(type) {
      if (type === 'basic') {
        this.form.session = {
          organisation: 'Default Org',
          scheduled_date: new Date(),
          location: 'Default Location',
          intermission_duration: { months: 0, days: 0, microseconds: 5 * 60 * 1_000_000 },
          static_at_end: false,
        };
        this.intermissionMinutes = 5;
        this.intermissionSeconds = 0;
        this.form.stations = [
          { title: 'Station A', index: 0, duration: { months: 0, days: 0, microseconds: 10 * 60 * 1_000_000 }, durationMinutes: 10 },
        ];
        this.form.slots = [
          { slot_time: 'AM', runs: [{ scheduled_start: '09:00', scheduled_end: '10:00' }], circuits: [{ key: 'A', female_only: false }] },
          { slot_time: 'PM', runs: [{ scheduled_start: '14:00', scheduled_end: '15:00' }], circuits: [{ key: 'A', female_only: false }] },
        ];
      } else if (type === 'advanced') {
        this.form.session = {
          organisation: 'Advanced Org',
          scheduled_date: new Date(),
          location: 'Advanced Location',
          intermission_duration: { months: 0, days: 0, microseconds: 10 * 60 * 1_000_000 },
          static_at_end: true,
        };
        this.intermissionMinutes = 10;
        this.intermissionSeconds = 0;
        this.form.stations = [
          { title: 'Station A', index: 0, duration: { months: 0, days: 0, microseconds: 15 * 60 * 1_000_000 }, durationMinutes: 15 },
          { title: 'Station B', index: 1, duration: { months: 0, days: 0, microseconds: 15 * 60 * 1_000_000 }, durationMinutes: 15 },
        ];
        this.form.slots = [
          { slot_time: 'AM', runs: [{ scheduled_start: '09:00', scheduled_end: '10:30' }], circuits: [{ key: 'A', female_only: false }, { key: 'B', female_only: true }] },
          { slot_time: 'PM', runs: [{ scheduled_start: '14:00', scheduled_end: '15:30' }], circuits: [{ key: 'A', female_only: false }, { key: 'B', female_only: true }] },
        ];
      }
      this.setDirty();
    },
  },
});

interface StationPayload {
  title: string;
  index: number;
  duration: { months: 0, days: 0, microseconds: number };
  durationMinutes: number;
}

interface CircuitPayload {
  key: string;
  female_only: boolean;
}

interface RunPayload {
  scheduled_start: string;
  scheduled_end: string;
}