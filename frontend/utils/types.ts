import { string } from "zod";
import { Time } from "@internationalized/date"
import type { CalendarDate, ZonedDateTime } from "@internationalized/date"

export type AuthBody = {
    access_token: string;
    token_type: string;
    username: string;
    role: string;
    organisation: string;
}

export type PgInterval = {
    months: number;
    days: number;
    microseconds: number;
}

export type PrimitiveSession = {
    id: string;
    organiser_id: string;
    organisation_id: string;
    scheduled_date: string;
    location: string;
    total_stations: number;
    feedback: boolean;
    feedback_duration: PgInterval | null;
    intermission_duration: PgInterval;
    static_at_end: boolean;
    status: string; // "new" | "prep" | "ready" | "pending" | "running" | "completed"
    created_at: string;
}

export interface ISession {
    id: string;
    organiser_id: string;
    organisation_id: string;
    scheduled_date: CalendarDate;
    location: string;
    total_stations: number;
    feedback: boolean;
    feedback_duration: PgInterval | null;
    intermission_duration: PgInterval;
    static_at_end: boolean;
    status: string; // "new" | "prep" | "ready" | "pending" | "running" | "completed"
    created_at: ZonedDateTime;
}

export interface ISessionInterval {
    name: string;
    duration: PgInterval | false;
}

export type SessionPayload = {
    scheduled_date: CalendarDate,
    location: string,
    intermission_duration: PgInterval,
    feedback: boolean,
    feedback_duration: PgInterval | null,
    static_at_end: boolean,
}

export interface IStation {
    id: string;
    session_id: string;
    title: string;
    index: number;
    duration: PgInterval;
}

export type StationPayload = {
    title: string;
    index: number;
    duration: PgInterval;
}

export interface IStationPayload {
    title: string;
    index: number;
    duration: PgInterval;
}

export type CircuitPayload = {
    female_only: boolean;
}

export interface IRunPayload {
    flip_allocation: boolean;
    scheduled_start: Time;
    scheduled_end: Time; // scheduled_end calculated on backend, removed when api calling
}

export type RunPayload = {
    flip_allocation: boolean;
    scheduled_start: Time;
    scheduled_end: Time; // scheduled_end calculated on backend, removed when api calling
}

export interface ISlotPayload {
    runs: RunPayload[];
    circuits: CircuitPayload[];
}

export type SlotPayload = {
    // instantiated as an array, index will be converted to "keys" on backend 
    runs: RunPayload[];
    circuits: CircuitPayload[];
}

export type TemplateSessionPayload = {
    name: string;
    feedback: boolean;
    feedback_duration: PgInterval | null;
    intermission_duration: PgInterval;
    static_at_end: boolean;
}

export type TemplateStationPayload = {
    title: string;
    index: number;
    duration: PgInterval;
}

export interface ITemplateStationPayload {
    title: string;
    index: number;
    duration: PgInterval;
}

export type TemplatePayload = {
    template_session: TemplateSessionPayload;
    template_stations: TemplateStationPayload[];
}

export type TemplateStation = {
    id: string;
    template_id: string;
    title: string;
    index: number;
    duration: PgInterval;
}

export type TemplateSessionWithStations = {
    id: string;
    name: string;
    feedback: boolean;
    feedback_duration: PgInterval | null;
    intermission_duration: PgInterval;
    static_at_end: boolean;
    stations: TemplateStation[];
}