import { defineStore } from "pinia"
import { apiFetch } from "~/composables/apiFetch"
import Session from "~/layouts/session.vue";

export const useSessionStore = defineStore("session", {
  state: () => ({
    session: null as Session,
    stations: [] as Station[],
    slots: [] as SlotStruct[],
    loading: false,
    error: null,
  }),
  actions: {
    async fetchSession(sessionId) {
      this.loading = true;
      this.error = null;
      try {
        const session_res = await apiFetch(`/sessions/get?id=${sessionId}`, {
            method: "GET",
        });
        this.session = session_res;
        console.log(session_res);
        const stations_res = await apiFetch(`/stations/get-session?id=${sessionId}`, {
            method: "GET",
        });
        this.stations = stations_res;
        console.log(stations_res);
        const slot_res = await apiFetch(`/slots/get-session?id=${sessionId}`, {
            method: "GET",
        });
        this.slots = [] as SlotStruct[];
        for (const slot of slot_res) {
            let slot_runs = await apiFetch(`/runs/get-slot?id=${slot.id}`, {
                method: "GET",
            });
            let slot_circuits = await apiFetch(`/circuits/get-slot?id=${slot.id}`, {
                method: "GET",
            });
            this.slots.push({
                data: slot,
                runs: slot_runs,
                circuits: slot_circuits,
            })
        }
        console.log(this.slots);
      } catch (error) {
        this.error = error.message || "Failed to fetch session";
        console.log(error);
        this.session = null as Session;
        this.stations = [] as Station[];
        this.slots = [] as SlotStruct[];
      } finally {
        this.loading = false;
      }
    },
    clearSession() {
      this.session = null as Session;
      this.stations = [] as Station[];
      this.slots = [] as SlotStruct[];
      this.loading = false;
      this.error = null;
    },
  },
});

interface Session {
    id: string,
    organiser_id: string,
    organisation: string,
    scheduled_date: string,
    location: string,
    total_stations: number,
    feedback: boolean,
    feedback_duration: Interval,
    intermission_duration: Interval,
    static_at_end: boolean,
    uploaded: boolean,
    allocated: boolean,
    created_at: string
}

interface Station {
    id: string,
    session_id: string,
    title: string,
    index: number,
    duration: Interval
}

interface Slot {
    id: string,
    session_id: string,
    key: string,
}

interface Circuit {
    id: string,
    session_id: string,
    slot_id: string,
    key: string,
    female_only: boolean,
    current_rotation: number,
    status: string,
    feedback: boolean,
    intermission: boolean,
    timer_start: string,
    timer_end: string,
}

interface Run {
    id: string,
    slot_id: string,
    flip_allocation: boolean,
    scheduled_start: string,
    scheduled_end: string,
    timer_start: string,
    timer_end: string,
}

interface Interval {
    months: number,
    days: number,
    microseconds: number
}

interface SlotStruct {
    data: Slot,
    runs: Run[],
    circuits: Circuit[],
}