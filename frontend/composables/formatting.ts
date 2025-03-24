export const formatInterval = (interval) => {
    if (!interval || typeof interval !== "object") return 0;

    const { months = 0, days = 0, microseconds = 0 } = interval;

    const daysFromMonths = months * 30;
    const totalDays = daysFromMonths + days;
    const minutesFromDays = totalDays * 24 * 60;
    const minutesFromMicroseconds = microseconds / (1_000_000 * 60);

    return Math.round(minutesFromDays + minutesFromMicroseconds) + " min"; // in minutes
};

export const formatDate = (date) => {
     return date ? new Date(date).toLocaleDateString() : "N/A";
};

// ISO 8601 to HH:MM
export const formatTimeFromISO = (isoString: string | null): string | null => {
    if (!isoString) return null;

    try {
        const date = new Date(isoString);
        if (isNaN(date.getTime())) return null;

        const hours = String(date.getUTCHours()).padStart(2, "0");
        const minutes = String(date.getUTCMinutes()).padStart(2, "0");
        return `${hours}:${minutes}`;
    } catch (error) {
        return null;
    }
};