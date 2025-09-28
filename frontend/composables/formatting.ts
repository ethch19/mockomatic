/**
 * Convert into minutes
 * @param interval
 * @returns Duration in minutes
*/
export const formatInterval = (interval: PgInterval) => {
    if (!interval) return 0;

    const daysFromMonths = interval.months * 30;
    const totalDays = daysFromMonths + interval.days;
    const minutesFromDays = totalDays * 24 * 60;
    const minutesFromMicroseconds = interval.microseconds / 60_000_000;

    return Math.round(minutesFromDays + minutesFromMicroseconds); // in minutes
};

export const formatDate = (date: string) => {
    return new Date(date).toLocaleDateString();
};

/**
 * ISO 8601 to HH:MM
 * @param isoString
 * @returns String formatted as HH:MM or null if invalid
*/
export const formatTimeFromISO = (isoString: string | null): string | null => {
    // ISO 8601 to HH:MM
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