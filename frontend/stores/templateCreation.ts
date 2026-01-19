import type { ITemplateStationPayload } from "~/utils/types";

export const useTemplateCreationStore = defineStore("templateCreation", {
    state: () => ({
        payload: {
            template_session: {
                name: "",
                feedback: false,
                feedback_duration: null,
                intermission_duration: { months: 0, days: 0, microseconds: 0 },
                static_at_end: false,
            } as TemplateSessionPayload,
            template_stations: [] as TemplateStationPayload[],
        } as TemplatePayload,
        isDirty: false, // track unsaved changes
        template_stepper: [
            { title: "Template Details", icon: "lucide:folder-pen" },
            { title: "Stations", icon: "lucide:gallery-vertical", },
            { title: "Review", icon: "lucide:file-check", }
        ],
    }),
    actions: {
        setDirty(dirty = true) {
            this.isDirty = dirty;
        },
        resetpayload() {
            this.payload = {
                template_session: {
                    name: "",
                    feedback: false,
                    feedback_duration: null,
                    intermission_duration: { months: 0, days: 0, microseconds: 0 },
                    static_at_end: false,
                } as TemplateSessionPayload,
                template_stations: [] as TemplateStationPayload[],
            } as TemplatePayload;
            this.isDirty = false;
        },
        async pushTemplate() {
            try {
                const response = await apiFetch("/templates/create", {
                    method: "POST",
                    body: this.payload,
                });
                return true;
            } catch (err) {
                toast.error("Failed to get : ", err);
                return false;
            }
        },
        addStation(title = "", seconds = 60) {
            const nextIndex = this.payload.template_stations.length;
            this.payload.template_stations = [...this.payload.template_stations,
                {
                    title,
                    index: nextIndex,
                    duration: { months: 0, days: 0, microseconds: seconds * 1_000_000 },
            }];
            this.setDirty();
        },
        updateStation(rowIndex: number, columnId: string, value: any) {
            if (this.payload.template_stations[rowIndex]) {
                this.payload.template_stations[rowIndex][columnId] = value;
                this.payload.template_stations = [...this.payload.template_stations];
            }
        },
        deleteSelectedStations(selected: Row<ITemplateStationPayload>[]) {
            const indexes = selected.map(row => row.original.index);
            this.payload.template_stations = this.payload.template_stations.filter(station => !indexes.includes(station.index));
            this.payload.template_stations.forEach((station, idx) => station.index = idx);
            this.setDirty();
        },
        reorderStation(rowIndex: number, targetRowIndex: number, instruction: string) {
            if (rowIndex === targetRowIndex) return;
            if (this.payload.template_stations.length <= rowIndex || this.payload.template_stations.length <= targetRowIndex) return;
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
            let newStations = this.payload.template_stations.toSpliced(rowIndex, 1);
            newStations.splice(newIndex, 0, this.payload.template_stations[rowIndex])
            newStations.forEach((station, index) => station.index = index);
            this.payload.template_stations = newStations
        },
        validateStations() {
            if (this.payload.template_stations.length > 0) {
                const con_station = this.payload.template_stations[0];
                const valid = this.payload.template_stations.every((value, index) => {
                    if (this.payload.template_session.static_at_end && index == this.payload.template_stations.length - 1) {
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
                return "Template must have at least 1 station!"
            }
        },
    },
});