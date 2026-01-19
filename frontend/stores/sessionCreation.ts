import { Time, toCalendarDateTime, toZoned, type DateValue, type TimeDuration, getLocalTimeZone } from "@internationalized/date"
import type { Row } from "@tanstack/vue-table";
import Stations from "~/pages/sessions/new/stations.vue";
import type { ISlotPayload, RunPayload, SlotPayload } from "~/utils/types";
import { getBase26Key, formatIntervalMicroseconds, formatMicroseconds } from "~/composables/formatting"
import { valueUpdater } from "~/lib/utils";
import { toast } from "vue-sonner";

// REACTIVITY NOTES TO SELF, specifically for TanStack
// https://github.com/TanStack/table/pull/5687#issuecomment-2281067245
// data is shadowRef, must mutate full data
// hence, when making any changes to an array (push, mutate) etc, you MUST create a new object and reassign it
// DO NOT MUTATE existing one, or reactivity will NOT work

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
        session_stepper: [
            { title: "Session Details", icon: "lucide:folder-pen" },
            { title: "Stations", icon: "lucide:gallery-vertical", },
            { title: "Timings", icon: "lucide:clock", },
            { title: "Review", icon: "lucide:file-check", }
        ],
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
        async pushSession() {
            try {
                const selected_date = this.payload.session.scheduled_date;
                const localTz = getLocalTimeZone();

                const formatted_payload = {
                    ...this.payload,
                    session: {
                        ...this.payload.session,
                        scheduled_date: this.payload.session.scheduled_date.toString(),
                    },
                    slots: this.payload.slots.map((slot) => {
                        return { 
                            circuits: slot.circuits,
                            runs: slot.runs.map((run) => {
                                const start_zoned = toZoned(toCalendarDateTime(selected_date, run.scheduled_start), localTz);
                                const end_zoned = toZoned(toCalendarDateTime(selected_date, run.scheduled_end), localTz);
                                
                                return {
                                    ...run,
                                    scheduled_start: start_zoned.toAbsoluteString(),
                                    scheduled_end: end_zoned.toAbsoluteString()
                                }
                            })
                        }
                    })
                }
                const response = await apiFetch("/sessions/create", {
                    method: "POST",
                    body: formatted_payload,
                });
                return true;
            } catch (err) {
                toast.error("Failed to create session: " + err);
                return false;
            }
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
                this.payload.stations = [...this.payload.stations];
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
            this.payload.stations = newStations
        },
        validateStations() {
            if (this.payload.stations.length > 0) {
                const con_station = this.payload.stations[0];
                const valid = this.payload.stations.every((value, index) => {
                    if (this.payload.session.static_at_end && index == this.payload.stations.length - 1) {
                        return true;
                    }
                    return value.duration.microseconds == con_station.duration.microseconds && value.duration.days == con_station.duration.days && value.duration.months == con_station.duration.months;
                });
                if (!valid) {
                    return "Stations do not have the same durations, try turning on static at end"
                } else {
                    return valid
                }
            } else {
                return "Session must have at least 1 station!"
            }
        },
        addSlot() {
            const slot_index = this.payload.slots.length;
            let slot_key = getBase26Key(slot_index);
            const new_slot = { key: slot_key, runs: [], circuits: [] } as SlotPayload;
            this.payload.slots = [...this.payload.slots, new_slot];
            this.addRun(slot_index);
            this.addCircuit(slot_index);
            this.setDirty();
            return slot_index;
        },
        deleteSlot(slot_index: number) {
            if (this.payload.slots[slot_index]) {
                this.payload.slots = this.payload.slots.filter((slot, idx) => idx != slot_index);
                this.setDirty();
            }
        },
        deleteSelectedSlots(selected: Row<ISlotPayload>[]) {
            const keys = selected.map(row => row.original.key);
            this.payload.slots = this.payload.slots.filter(slot => !keys.includes(slot.key));
            this.payload.slots = this.payload.slots.map((slot, idx) => {
                return {
                    ...slot,
                    key: getBase26Key(idx)
                };
            });
            this.setDirty();
        },
        addCircuit(slot_index: number) {
            if (this.payload.slots[slot_index]) {
                this.payload.slots[slot_index].circuits = [...this.payload.slots[slot_index].circuits, { female_only: false }];
                this.setDirty();
            }
        },
        toggleCircuit(slot_index: number, circuit_index: number) {
            if (this.payload.slots[slot_index] && this.payload.slots[slot_index].circuits[circuit_index]) {
                this.payload.slots[slot_index].circuits[circuit_index].female_only = !this.payload.slots[slot_index].circuits[circuit_index].female_only;
                this.payload.slots[slot_index].circuits = [...this.payload.slots[slot_index].circuits];
                this.setDirty();
            }
        },
        removeCircuit(slot_index: number, circuit_index: number) {
            if (this.payload.slots[slot_index].circuits.length == 1) {
                return
            } // must be at least 1 circuit in each slot
            if (this.payload.slots[slot_index] && this.payload.slots[slot_index].circuits[circuit_index]) {
                this.payload.slots[slot_index].circuits = this.payload.slots[slot_index].circuits.filter((circuit, idx) => idx != circuit_index);
                this.setDirty();
            }
        },
        addRun(slot_index: number) {
            const slot: SlotPayload = this.payload.slots[slot_index];
            if (slot) {
                const slot_runs = slot.runs.length;
                const total_duration = Math.floor(this.calculateRunDuration() / 1000); // in milliseconds
                if (slot_runs > 0) {
                    const scheduled_start = slot.runs[slot_runs-1].scheduled_end.copy();
                    this.payload.slots[slot_index].runs = [...this.payload.slots[slot_index].runs, { scheduled_start, scheduled_end: scheduled_start.add({ milliseconds: total_duration}), flip_allocation: false}];
                } else {
                    if (slot_index > 0) {
                        let index = slot_index - 1;
                        let pre_slot: SlotPayload = this.payload.slots[index];
                        let scheduled_start: Time | null = null;
                        while (index >= 0) {
                            pre_slot = this.payload.slots[index];
                            if (pre_slot) {
                                if (pre_slot.runs.length > 0) {
                                    scheduled_start = pre_slot.runs[pre_slot.runs.length-1].scheduled_end
                                    break;
                                }
                            }
                            index -= 1;
                        }
                        if (scheduled_start == null) {
                            this.payload.slots[slot_index].runs = [...this.payload.slots[slot_index].runs, { scheduled_start: new Time(8), scheduled_end: new Time(8).add({ milliseconds: total_duration }), flip_allocation: false}];
                        } else {
                            this.payload.slots[slot_index].runs = [...this.payload.slots[slot_index].runs, { scheduled_start, scheduled_end: scheduled_start.add({ milliseconds: total_duration }), flip_allocation: false}];
                        }
                    } else {
                        this.payload.slots[slot_index].runs = [...this.payload.slots[slot_index].runs, { scheduled_start: new Time(8), scheduled_end: new Time(8).add({ milliseconds: total_duration }), flip_allocation: false}];
                    }
                }
                const new_run_index = this.payload.slots[slot_index].runs.length;
                for (let i = slot_index + 1; i < this.payload.slots.length; i++) {
                    const runs_length = this.payload.slots[i].runs.length;
                    for (let j = 0; j < runs_length; j++) {
                        const current_run: RunPayload = this.payload.slots[i].runs[j];
                        const new_run: RunPayload = {
                            scheduled_start: current_run.scheduled_start.add({ milliseconds: total_duration }),
                            scheduled_end: current_run.scheduled_end.add({ milliseconds: total_duration }),
                            flip_allocation: current_run.flip_allocation,
                        }
                        this.payload.slots[i].runs = this.payload.slots[i].runs.filter((run, idx) => idx != j);
                        this.payload.slots[i].runs = [...this.payload.slots[i].runs, new_run];
                    }
                }
                this.payload.slots = [...this.payload.slots];
                this.setDirty();
            }
        },
        recalculateTimings() {
            if (this.payload.slots.length == 0) {
                return
            }
            const run_duration = this.calculateRunDuration();
            const run_time = formatMicroseconds(run_duration);
            for (let i = 0; i < this.payload.slots.length; i++) {
                for (let j = 0; j < this.payload.slots[i].runs.length; j++) {
                    let current_run = this.payload.slots[i].runs[j];
                    const cur_duration = current_run.scheduled_end.subtract({
                        hours: current_run.scheduled_start.hour,
                        minutes: current_run.scheduled_start.minute,
                        seconds: current_run.scheduled_start.second,
                        milliseconds: current_run.scheduled_start.millisecond
                    });
                    if (run_time.compare(cur_duration) != 0) {
                        const diff_duration: TimeDuration = {
                            hours: run_time.hour - cur_duration.hour,
                            minutes: run_time.minute - cur_duration.minute,
                            seconds: run_time.second - cur_duration.second,
                            milliseconds: run_time.millisecond - cur_duration.millisecond
                        }
                        this.payload.slots[i].runs = this.payload.slots[i].runs.map((run, index) => {
                            if (index == j) {
                                return {
                                    scheduled_start: run.scheduled_start, 
                                    scheduled_end: run.scheduled_end.add(diff_duration),
                                    flip_allocation: run.flip_allocation,
                                };
                            }
                            return run
                        });
                        if (j + 1 >= this.payload.slots[i].runs.length) {
                            if (i + 1 >= this.payload.slots.length) {
                                continue;
                            } else {
                                this.onRunTimeChanged(i+1, 0, "scheduled_start", diff_duration);
                            }
                        } else {
                            this.onRunTimeChanged(i, j+1, "scheduled_start", diff_duration);
                        }
                    }
                }
            }
        },
        deleteRun(slot_index: number, run_index: number) {
            if (this.payload.slots[slot_index].runs.length == 1) {
                return;
            }
            if (this.payload.slots[slot_index] && this.payload.slots[slot_index].runs[run_index]) {
                this.payload.slots[slot_index].runs = this.payload.slots[slot_index].runs.filter((run, idx) => idx != run_index);
                this.setDirty();
            }
        },
        calculateRunDuration() { // run duration in microseconds
            const stations_length: number = this.payload.stations.length;
            const total_station_durations: number = this.payload.stations.reduce((acc, value) => acc + formatIntervalMicroseconds(value.duration), 0); // in microseconds
            const total_duration: number = this.payload.session.feedback ? total_station_durations + (formatIntervalMicroseconds(this.payload.session.feedback_duration) + formatIntervalMicroseconds(this.payload.session.intermission_duration)) * stations_length : total_station_durations + formatIntervalMicroseconds(this.payload.session.intermission_duration) * stations_length;
            return total_duration; // microseconds
        },
        onRunTimeChanged(slot_index: number, run_index: number, time_field: string, new_duration: TimeDuration) {
            if (this.payload.slots[slot_index] && this.payload.slots[slot_index].runs[run_index]){
                let slot_start: number = 0;
                let run_start: number = 0;
                if (time_field == "scheduled_start") {
                    let last_run: RunPayload | null = null;
                    if (run_index != 0) {
                        last_run = this.payload.slots[slot_index].runs[run_index - 1];
                    } else {
                        if (slot_index != 0) {
                            last_run = this.payload.slots[slot_index-1].runs[this.payload.slots[slot_index-1].runs.length - 1];
                        } else {
                            last_run = this.payload.slots[slot_index].runs[run_index];
                        }
                    }
                    if (last_run?.scheduled_end.compare(this.payload.slots[slot_index].runs[run_index].scheduled_start.add(new_duration)) <= 0) {
                        slot_start = slot_index;
                        run_start = run_index;
                    } else {
                    }
                }
                for (let i = slot_start; i < this.payload.slots.length; i++) {
                    this.payload.slots[i].runs = this.payload.slots[i].runs.map((value, idx) => {
                        if (idx >= run_index && i == slot_start) {
                            return {
                                scheduled_start: value.scheduled_start.add(new_duration),
                                scheduled_end: value.scheduled_end.add(new_duration),
                                flip_allocation: value.flip_allocation,
                            }
                        } else if(i != slot_start) {
                            return {
                                scheduled_start: value.scheduled_start.add(new_duration),
                                scheduled_end: value.scheduled_end.add(new_duration),
                                flip_allocation: value.flip_allocation,
                            }
                        } else {
                            return value
                        }
                    })
                }
                this.payload.slots = [...this.payload.slots];
            }
        },
    },
});