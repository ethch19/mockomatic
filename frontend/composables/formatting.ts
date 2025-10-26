import { Time } from "@internationalized/date";

/**
 * @param interval
 * @returns Duration in MIN:SECONDS
*/
export const formatInterval = (interval: PgInterval) =>  {
    if (!interval) return null;

    const daysFromMonths = interval.months * 30;
    const totalDays = daysFromMonths + interval.days;
    const minutesFromDays = totalDays * 24 * 60;
    if (interval.microseconds % 60_000_000 != 0) {
        const minutesFromMicroseconds = Math.floor(interval.microseconds / 60_000_000);
        const secondsFromMicroseconds = Math.floor((interval.microseconds % 60_000_000) / 1_000_000);
        
        const formattedMinutes = String(minutesFromDays + minutesFromMicroseconds).padStart(2, '0');
        const formattedSecond = String(secondsFromMicroseconds).padStart(2, '0');
        return `${formattedMinutes}:${formattedSecond}`
    }
    const minutesFromMicroseconds = interval.microseconds / 60_000_000;
    const formattedMinutes = String(minutesFromDays + minutesFromMicroseconds).padStart(2, '0');

    return `${formattedMinutes}:00`
};


/**
 * Convert microseconds into Time/Duration
 * @param number (microseconds)
 * @returns Time
*/
export const formatMicroseconds = (in_microseconds: number) => {
    let new_time = {
        hour: 0,
        minute: 0,
        second: 0,
        millisecond: 0,
    };
    new_time.hour = Math.floor(in_microseconds / 3_600_000_000);
    in_microseconds = in_microseconds % 3_600_000_000;
    new_time.minute = Math.floor(in_microseconds / 60_000_000);
    in_microseconds = in_microseconds % 60_000_000;
    new_time.second = Math.floor(in_microseconds / 1_000_000);
    in_microseconds = in_microseconds % 1_000_000;
    new_time.millisecond = Math.floor(in_microseconds / 1_000);

    return new Time(new_time.hour, new_time.minute, new_time.second, new_time.millisecond)
}

/**
 * @param interval
 * @returns number in microseconds
*/
export const formatIntervalMicroseconds = (interval: PgInterval) =>  {
    if (!interval) return null;

    const daysFromMonths = interval.months * 30;
    const totalDays = daysFromMonths + interval.days;
    const microsecondsFromDays = totalDays * 86_400_000_000;

    return microsecondsFromDays + interval.microseconds;
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

/**
 * Handles incomplete input, adds leading zeros, and clamps invalid values.
 * @param string
 * @returns ISO 8601 string, false if null/undefined value given
*/
export function normalizeTimeString(value: string | null | undefined): string | boolean {
    if (!value) return false;

    const cleaned = value.replace(/[^0-9:]/g, '');
    const parts = cleaned.split(':');

    let [hourStr = '0', minuteStr = '0', secondStr = '0'] = parts;

    // "08:030:00" -> "08:03:00" by taking only the first 2 digits
    hourStr = hourStr.slice(0, 2);
    minuteStr = minuteStr.slice(0, 2);
    secondStr = secondStr.slice(0, 2);

    let hour = parseInt(hourStr, 10) || 0;
    let minute = parseInt(minuteStr, 10) || 0;
    let second = parseInt(secondStr, 10) || 0;

    // Clamp values to valid time ranges (e.g., "99:00:00" -> "23:00:00")
    hour = Math.max(0, Math.min(23, hour));
    minute = Math.max(0, Math.min(59, minute));
    second = Math.max(0, Math.min(59, second));

    // Format back to string with leading zeros
    const formattedHour = String(hour).padStart(2, '0');
    const formattedMinute = String(minute).padStart(2, '0');
    const formattedSecond = String(second).padStart(2, '0');

    return `${formattedHour}:${formattedMinute}:${formattedSecond}`;
}

export function normalizeMinuteSeconds(value: string | null | undefined): string | boolean {
    if (!value) return false;

    const cleaned = value.replace(/[^0-9:]/g, '');
    const parts = cleaned.split(':');

    let [minuteStr = '0', secondStr = '0'] = parts;
    secondStr = secondStr.slice(0, 2);

    let minute = parseInt(minuteStr, 10) || 0;
    let second = parseInt(secondStr, 10) || 0;

    if (second > 59) {
        minute += Math.floor(second / 60);
        second = second % 60;
    }

    // Format back to string with leading zeros
    const formattedMinute = String(minute).padStart(2, '0');
    const formattedSecond = String(second).padStart(2, '0');

    return `${formattedMinute}:${formattedSecond}`;
}

/**
 * Converts string in format of MM:SS into PgInterval
 * @param string
 * @returns PgInterval
*/
export function formatIntervalFromString(value: string): PgInterval {
    const parts = value.split(':');
    const [minuteStr = '0', secondStr = '0'] = parts;
    const minute = parseInt(minuteStr, 10) || 0;
    const second = parseInt(secondStr, 10) || 0;

    const microseconds = Math.round((minute * 60 + second) * 1_000_000);

    return { months: 0, days: 0, microseconds }
}

export const getBase26Key = (index: number): string => {
    let n = index + 1;
    let key = "";
    while (n > 0) {
        const remainder = (n - 1) % 26;
        key = String.fromCharCode(65 + remainder) + key;
        n = Math.floor((n - 1) / 26);
    }
    return key;
};