import { getLocalTimeZone, now, parseAbsoluteToLocal, parseDate, Time, ZonedDateTime } from "@internationalized/date"
import { toast } from "vue-sonner";
import type { PrimitiveSession } from "~/utils/types";

export const useSessionBrowserStore = defineStore("sessionBrowser", {
    state: () => ({
        sessions: [] as ISession[],
        last_updated: null as ZonedDateTime,
    }),
    actions: {
        async fetchAll() {
            try {
                const response: PrimitiveSession[]  = await apiFetch("/sessions/get-all");
                this.sessions = response.map((session) => ({
                    ...session,
                    scheduled_date: parseDate(session.scheduled_date),
                    created_at: parseAbsoluteToLocal(session.created_at),
                }));
                this.last_updated = now(getLocalTimeZone());
            } catch (err) {
                toast.error("Failed to get sessions: " + err);
            }
        },
        async deleteSession(id: string) {
            try {
                const response = await apiFetch("/sessions/delete", {
                    method: "POST",
                    body: { ids: [id] }
                });
                toast.success("Deleted session (" + id + ") successfully")
                this.sessions = this.sessions.filter((session) => session.id != id);
                this.last_updated = now(getLocalTimeZone());
            } catch (err) {
                console.log(err);
                toast.error("Failed to delete session (" + id + ")")
            }
        },
    },
});