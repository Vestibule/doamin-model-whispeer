import { invoke } from "@tauri-apps/api/core";

export interface OrchestrateResult {
  markdown: string;
  mermaid: string;
  model: DomainModel;
}

export interface DomainModel {
  entities: Entity[];
  relations: Relation[];
  invariants: Invariant[];
}

export interface Entity {
  id: string;
  name: string;
  description?: string;
  attributes: Attribute[];
  primaryKey?: string[];
  uniqueConstraints?: UniqueConstraint[];
}

export interface Attribute {
  name: string;
  type: string;
  description?: string;
  required?: boolean;
  unique?: boolean;
  defaultValue?: any;
  validation?: AttributeValidation;
}

export interface AttributeValidation {
  minLength?: number;
  maxLength?: number;
  min?: number;
  max?: number;
  pattern?: string;
  enum?: any[];
  custom?: string;
}

export interface UniqueConstraint {
  name: string;
  attributes: string[];
}

export interface Relation {
  id: string;
  name: string;
  description?: string;
  from: RelationEnd;
  to: RelationEnd;
  cardinality: Cardinality;
  required?: boolean;
  cascadeDelete?: boolean;
}

export interface RelationEnd {
  entityId: string;
  attribute?: string;
  label?: string;
}

export interface Cardinality {
  from: string;
  to: string;
}

export interface Invariant {
  id: string;
  name: string;
  description?: string;
  type: string;
  scope?: InvariantScope;
  expression: string;
  severity?: string;
  errorMessage?: string;
}

export interface InvariantScope {
  entities?: string[];
  relations?: string[];
}

export interface TranscriptionResult {
  text: string;
  language: string | null;
  duration_ms: number;
}

export interface AudioDevice {
  name: string;
  is_default: boolean;
}

/**
 * Orchestrate the entire flow: transcript -> domain model -> markdown + mermaid
 * @param transcript - The input transcript to process
 * @returns The orchestrated result with markdown, mermaid, and domain model
 */
export async function orchestrate(transcript: string): Promise<OrchestrateResult> {
  return invoke<OrchestrateResult>("orchestrate", { transcript });
}

/**
 * Start recording audio with voice activity detection (backend)
 * @returns Status message indicating where files will be saved
 */
export async function startRecording(): Promise<string> {
  return invoke<string>("start_recording");
}

/**
 * Stop recording audio
 * @returns Status message
 */
export async function stopRecording(): Promise<string> {
  return invoke<string>("stop_recording");
}

/**
 * Transcribe an audio file using Whisper (NOT YET IMPLEMENTED)
 * @param audioPath - Path to the audio file (WAV format)
 * @returns Error message - use Web Speech API instead
 */
export async function transcribeAudio(audioPath: string): Promise<string> {
  return invoke<string>("transcribe_audio", { audioPath });
}

/**
 * List available audio input devices
 * @returns Array of audio devices with their names and default status
 */
export async function listAudioDevices(): Promise<AudioDevice[]> {
  return invoke<AudioDevice[]>("list_audio_devices");
}

/**
 * Set the audio input device to use for recording
 * @param deviceName - Name of the audio device to use
 * @returns Success message
 */
export async function setAudioDevice(deviceName: string): Promise<string> {
  return invoke<string>("set_audio_device", { deviceName });
}
