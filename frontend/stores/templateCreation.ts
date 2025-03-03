import { defineStore } from "pinia";

export const useTemplateCreationStore = defineStore("templateCreation", {
  state: () => ({
    template: {
      name: "",
      feedback: false,
      feedback_duration: null,
      intermission_duration: 0,
      static_at_end: false,
    },
    stations: [] as StationMinutesPayload[],
    isDirty: false,
  }),
  actions: {
    setDirty(dirty = true) {
      this.isDirty = dirty;
    },
    resetForm() {
      this.form = {
        template: {
          name: "",
          feedback: false,
          feedback_duration: null,
          intermission_duration: 0,
          static_at_end: false,
        },
        stations: [] as StationMinutesPayload[],
        isDirty: false,
      };
      this.isDirty = false;
    },
    addStation(title = "", minutes = 1) {
      const nextIndex = this.stations.length;
      this.stations.push({
        title,
        index: nextIndex,
        duration: minutes,
      });
      this.setDirty();
    },
    removeStations(selected_stations) {
      if (selected_stations.length == this.stations.length) {
        this.stations = [] as StationMinutesPayload[];
        return;
      }
      const newStations = [...this.stations];
      for (let i = 0; i < selected_stations.length; i++) {
        const index = newStations.indexOf(selected_stations[i]);
        if (index > -1) {
          newStations.splice(index, 1);
        }
      }
      newStations.forEach((station, index) => station.index = index);
      this.stations = newStations;
    },
    onRowReorder(event) {
      const newStations = [...this.stations];
      newStations.splice(event.dragIndex, 1);
      newStations.splice(event.dropIndex, 0, this.stations[event.dragIndex]);
      newStations.forEach((station, index) => station.index = index);
      this.stations = newStations;
      this.setDirty();
    },
    formatDuration() {
      if (feedback) {
        return {
          ...this.template,
          feedback_duration: {
            months: 0,
            days: 0,
            microseconds: this.template.feedback_duration * 1_000_000,
          },
          intermission_duration: {
            months: 0,
            days: 0,
            microseconds: this.template.intermission_duration * 1_000_000,
          },
        }
      }
      return {
        ...this.template,
        intermission_duration: {
          months: 0,
          days: 0,
          microseconds: this.template.intermission_duration * 1_000_000,
        },
      }
    },
    formatStations() {
      let stations = [];
      for (let i = 0; i < this.stations.length; ++i) {
        const e = this.stations[i];
        stations.push({
          ...e,
          duration: {
            months: 0,
            days: 0,
            microseconds: e.duration * 60 * 1_000_000,
          }
        });
      }
      return stations;
    },
  },
});

interface StationMinutesPayload {
  title: string;
  index: number;
  duration: number;
}

interface StationPayload {
  title: string;
  index: number;
  duration: { months: 0, days: 0, microseconds: number };
}