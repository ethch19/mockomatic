export const useTemplateCreationStore = defineStore("templateCreation", {
    state: () => ({
        payload: {
            session: {
                name: "",
                feedback: false,
                feedback_duration: null,
                intermission_duration: { months: 0, days: 0, microseconds: 0 },
                static_at_end: false,
            } as TemplateSessionPayload,
            stations: [] as TemplateStationPayload[],
        },
        isDirty: false,
    }),
    actions: {
        setDirty(dirty = true) {
            this.isDirty = dirty;
        },
        resetForm() {
            this.payload = {
                session: {
                    name: "",
                    feedback: false,
                    feedback_duration: null,
                    intermission_duration: { months: 0, days: 0, microseconds: 0 },
                    static_at_end: false,
                } as TemplateSessionPayload,
                stations: [] as TemplateStationPayload[],
            }
            this.isDirty = false;
        },
        addStation(title = "", seconds = 60) {
            const nextIndex = this.payload.stations.length;
            this.payload.stations.push({
                title,
                index: nextIndex,
                duration: { months: 0, days: 0, microseconds: seconds * 1_000_000 },
            });
            this.setDirty();
        },
    },
});