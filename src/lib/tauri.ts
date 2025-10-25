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

// Interview types
export interface InterviewUserAnswer {
  section_id: number;
  question_index: number;
  question: string;
  answer: string;
}

export interface InterviewSection {
  section_id: number;
  section_title: string;
  answers: InterviewUserAnswer[];
}

export interface SectionCanvasResult {
  section_id: number;
  section_title: string;
  canvas_content: string;
}

export interface FullCanvasResult {
  markdown: string;
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

/**
 * Process interview section answers through LLM to generate canvas content
 * @param section - Interview section with user answers
 * @returns Canvas content for this section
 */
export async function processInterviewSection(
  section: InterviewSection
): Promise<SectionCanvasResult> {
  return invoke<SectionCanvasResult>("process_interview_section", { section });
}

/**
 * Generate the complete canvas markdown from all processed sections
 * @param sections - Array of processed section results
 * @returns Complete canvas markdown document
 */
export async function generateFullCanvas(
  sections: SectionCanvasResult[]
): Promise<FullCanvasResult> {
  return invoke<FullCanvasResult>("generate_full_canvas", { sections });
}

/**
 * Save interview state to a markdown file named after the project
 * @param projectName - Name of the project
 * @param stateJson - JSON string of the interview state
 * @returns Success message with file path
 */
export async function saveInterviewState(
  projectName: string,
  stateJson: string
): Promise<string> {
  return invoke<string>("save_interview_state", { projectName, stateJson });
}

/**
 * Load interview state from a JSON file
 * @param projectName - Name of the project
 * @returns JSON string of the saved state
 */
export async function loadInterviewState(
  projectName: string
): Promise<string> {
  return invoke<string>("load_interview_state", { projectName });
}

/**
 * List all saved project names
 * @returns Array of project names
 */
export async function listSavedProjects(): Promise<string[]> {
  return invoke<string[]>("list_saved_projects");
}

/**
 * Save canvas markdown to a file
 * @param projectName - Name of the project
 * @param markdown - Canvas markdown content
 * @returns Success message with file path
 */
export async function saveCanvasMarkdown(
  projectName: string,
  markdown: string
): Promise<string> {
  return invoke<string>("save_canvas_markdown", { projectName, markdown });
}
