<script lang="ts">
  import { Button, Card, Textarea, Badge, Heading, Spinner, Input, Select } from 'flowbite-svelte';
  import { ChevronRightOutline, ChevronLeftOutline, CheckCircleSolid, FloppyDiskOutline, FolderOpenOutline } from 'flowbite-svelte-icons';
  import { INTERVIEW_SECTIONS, type InterviewState, type UserAnswer } from './types/interview';
  import AudioInput from './AudioInput.svelte';
  import CanvasViewer from './CanvasViewer.svelte';
  import { processInterviewSection, generateFullCanvas, saveInterviewState, loadInterviewState, listSavedProjects, saveCanvasMarkdown, type InterviewSection as TauriInterviewSection, type SectionCanvasResult } from './tauri';
  import { onMount } from 'svelte';

  let projectName = $state("");
  let savedProjects = $state<string[]>([]);
  let selectedProject = $state("");
  let loading = $state(false);
  let interviewState = $state<InterviewState>({
    currentSection: 0,
    currentQuestionIndex: 0,
    answers: [],
    canvas: {},
    isComplete: false
  });

  let currentAnswer = $state("");
  let sections = $state([...INTERVIEW_SECTIONS]);
  let processedSections = $state<SectionCanvasResult[]>([]);
  let fullCanvasMarkdown = $state("");
  let processing = $state(false);
  let generatingCanvas = $state(false);
  let saving = $state(false);
  let savingCanvas = $state(false);
  let saveStatus = $state("");
  let canvasSaveStatus = $state("");
  let error = $state("");

  // Computed values
  let currentSection = $derived(sections[interviewState.currentSection]);
  let currentQuestion = $derived(
    currentSection?.questions[interviewState.currentQuestionIndex] || ""
  );
  let progress = $derived(
    Math.round(((interviewState.currentSection * 100) + 
    (interviewState.currentQuestionIndex / currentSection.questions.length * 100)) / sections.length)
  );
  let canGoNext = $derived(currentAnswer.trim().length > 0);
  let canGoPrevious = $derived(
    interviewState.currentSection > 0 || interviewState.currentQuestionIndex > 0
  );

  function handleAnswerSubmit(text: string) {
    currentAnswer = text;
    saveAndNext();
  }

  onMount(async () => {
    try {
      savedProjects = await listSavedProjects();
    } catch (e) {
      console.error("Failed to load saved projects:", e);
    }
  });

  async function saveState() {
    if (!projectName.trim()) return;
    
    saving = true;
    saveStatus = "";
    
    try {
      const stateToSave = {
        projectName,
        answers: interviewState.answers,
        sections: sections.map(s => ({ id: s.id, title: s.title, completed: s.completed })),
        currentSection: interviewState.currentSection,
        currentQuestionIndex: interviewState.currentQuestionIndex
      };
      
      const message = await saveInterviewState(projectName, JSON.stringify(stateToSave));
      saveStatus = message;
      
      // Refresh saved projects list
      savedProjects = await listSavedProjects();
      
      // Clear status after 3 seconds
      setTimeout(() => { saveStatus = ""; }, 3000);
    } catch (e) {
      error = `Erreur lors de la sauvegarde: ${String(e)}`;
    } finally {
      saving = false;
    }
  }

  async function loadState() {
    if (!selectedProject) return;
    
    loading = true;
    error = "";
    
    try {
      const stateJson = await loadInterviewState(selectedProject);
      const loadedState = JSON.parse(stateJson);
      
      // Restore state
      projectName = loadedState.projectName || selectedProject;
      interviewState.answers = loadedState.answers || [];
      interviewState.currentSection = loadedState.currentSection || 0;
      interviewState.currentQuestionIndex = loadedState.currentQuestionIndex || 0;
      
      // Restore section completion status
      if (loadedState.sections) {
        loadedState.sections.forEach((savedSection: any) => {
          const section = sections.find(s => s.id === savedSection.id);
          if (section) {
            section.completed = savedSection.completed;
          }
        });
      }
      
      // Load current answer
      const currentAnswerData = interviewState.answers.find(
        a => a.sectionId === currentSection.id && 
             a.questionIndex === interviewState.currentQuestionIndex
      );
      currentAnswer = currentAnswerData ? currentAnswerData.answer : "";
      
      saveStatus = `✓ Projet "${projectName}" chargé`;
      setTimeout(() => { saveStatus = ""; }, 3000);
    } catch (e) {
      error = `Erreur lors du chargement: ${String(e)}`;
    } finally {
      loading = false;
    }
  }

  async function saveAndNext() {
    if (!canGoNext) return;

    // Find if we already have an answer for this question (update case)
    const existingIndex = interviewState.answers.findIndex(
      a => a.sectionId === currentSection.id && 
           a.questionIndex === interviewState.currentQuestionIndex
    );

    const answer: UserAnswer = {
      sectionId: currentSection.id,
      questionIndex: interviewState.currentQuestionIndex,
      question: currentQuestion,
      answer: currentAnswer,
      timestamp: new Date()
    };
    
    if (existingIndex >= 0) {
      // Update existing answer
      interviewState.answers[existingIndex] = answer;
    } else {
      // Add new answer
      interviewState.answers.push(answer);
    }

    // Move to next question
    if (interviewState.currentQuestionIndex < currentSection.questions.length - 1) {
      interviewState.currentQuestionIndex++;
    } else {
      // Section completed - process it with LLM
      await processSectionWithLLM();
      
      // Mark section as completed
      sections[interviewState.currentSection].completed = true;
      
      // Move to next section
      if (interviewState.currentSection < sections.length - 1) {
        interviewState.currentSection++;
        interviewState.currentQuestionIndex = 0;
      } else {
        // Interview complete
        interviewState.isComplete = true;
      }
    }

    // Load the answer for the new question if it exists
    const nextAnswer = interviewState.answers.find(
      a => a.sectionId === currentSection.id && 
           a.questionIndex === interviewState.currentQuestionIndex
    );
    currentAnswer = nextAnswer ? nextAnswer.answer : "";

    // Auto-save state
    await saveState();
  }

  async function processSectionWithLLM() {
    processing = true;
    error = "";

    try {
      // Get all answers for current section
      const sectionAnswers = interviewState.answers.filter(
        a => a.sectionId === currentSection.id
      );

      // Convert to Tauri format
      const tauriSection: TauriInterviewSection = {
        section_id: currentSection.id,
        section_title: currentSection.title,
        answers: sectionAnswers.map(a => ({
          section_id: a.sectionId,
          question_index: a.questionIndex,
          question: a.question,
          answer: a.answer
        }))
      };

      // Process with LLM
      const result = await processInterviewSection(tauriSection);
      processedSections.push(result);
    } catch (e) {
      error = `Erreur lors du traitement de la section: ${String(e)}`;
    } finally {
      processing = false;
    }
  }

  async function handleGenerateCanvas() {
    generatingCanvas = true;
    error = "";

    try {
      const result = await generateFullCanvas(processedSections);
      fullCanvasMarkdown = result.markdown;
    } catch (e) {
      error = `Erreur lors de la génération du canvas: ${String(e)}`;
    } finally {
      generatingCanvas = false;
    }
  }

  async function saveCanvasPreview() {
    if (!projectName.trim() || processedSections.length === 0) return;
    
    savingCanvas = true;
    canvasSaveStatus = "";
    error = "";
    
    try {
      // Build partial canvas markdown from processed sections
      let markdown = "# Canvas — Rich Domain Model (DDD)\n\n";
      markdown += "> Objectif : cadrer un domaine avec un modèle riche (entités porteuses de logique, invariants explicites, langage ubiquiste). Remplis court et concret.\n\n";
      markdown += "---\n\n";
      
      for (const section of processedSections) {
        markdown += `## ${section.section_title}\n\n`;
        markdown += section.canvas_content;
        markdown += "\n\n";
      }
      
      const message = await saveCanvasMarkdown(projectName, markdown);
      canvasSaveStatus = "✓ Aperçu du canvas sauvegardé";
      
      // Clear status after 3 seconds
      setTimeout(() => { canvasSaveStatus = ""; }, 3000);
    } catch (e) {
      error = `Erreur lors de la sauvegarde du canvas: ${String(e)}`;
    } finally {
      savingCanvas = false;
    }
  }

  async function saveFullCanvas() {
    if (!projectName.trim() || !fullCanvasMarkdown) return;
    
    savingCanvas = true;
    canvasSaveStatus = "";
    error = "";
    
    try {
      const message = await saveCanvasMarkdown(projectName, fullCanvasMarkdown);
      canvasSaveStatus = "✓ Canvas complet sauvegardé";
      
      // Clear status after 3 seconds
      setTimeout(() => { canvasSaveStatus = ""; }, 3000);
    } catch (e) {
      error = `Erreur lors de la sauvegarde du canvas: ${String(e)}`;
    } finally {
      savingCanvas = false;
    }
  }

  async function goToPrevious() {
    if (!canGoPrevious) return;

    // Save current answer before moving
    if (currentAnswer.trim().length > 0) {
      const existingIndex = interviewState.answers.findIndex(
        a => a.sectionId === currentSection.id && 
             a.questionIndex === interviewState.currentQuestionIndex
      );

      const answer: UserAnswer = {
        sectionId: currentSection.id,
        questionIndex: interviewState.currentQuestionIndex,
        question: currentQuestion,
        answer: currentAnswer,
        timestamp: new Date()
      };
      
      if (existingIndex >= 0) {
        interviewState.answers[existingIndex] = answer;
      } else {
        interviewState.answers.push(answer);
      }
    }

    if (interviewState.currentQuestionIndex > 0) {
      interviewState.currentQuestionIndex--;
    } else if (interviewState.currentSection > 0) {
      interviewState.currentSection--;
      interviewState.currentQuestionIndex = sections[interviewState.currentSection].questions.length - 1;
    }

    // Load previous answer if it exists
    const previousAnswer = interviewState.answers.find(
      a => a.sectionId === currentSection.id && 
           a.questionIndex === interviewState.currentQuestionIndex
    );
    currentAnswer = previousAnswer ? previousAnswer.answer : "";

    // Auto-save state
    await saveState();
  }

  async function jumpToSection(sectionIndex: number) {
    // Save current answer before jumping
    if (currentAnswer.trim().length > 0) {
      const existingIndex = interviewState.answers.findIndex(
        a => a.sectionId === currentSection.id && 
             a.questionIndex === interviewState.currentQuestionIndex
      );

      const answer: UserAnswer = {
        sectionId: currentSection.id,
        questionIndex: interviewState.currentQuestionIndex,
        question: currentQuestion,
        answer: currentAnswer,
        timestamp: new Date()
      };
      
      if (existingIndex >= 0) {
        interviewState.answers[existingIndex] = answer;
      } else {
        interviewState.answers.push(answer);
      }
    }

    interviewState.currentSection = sectionIndex;
    interviewState.currentQuestionIndex = 0;
    
    // Load the answer for the new section's first question if it exists
    const targetAnswer = interviewState.answers.find(
      a => a.sectionId === sections[sectionIndex].id && 
           a.questionIndex === 0
    );
    currentAnswer = targetAnswer ? targetAnswer.answer : "";

    // Auto-save state
    await saveState();
  }
</script>

<div class="h-full w-full flex flex-col gap-4 p-6 bg-gray-50 dark:bg-gray-900">
  <!-- Project name header -->
  <Card class="flex-none">
    <div class="space-y-4">
      <div class="flex items-center gap-4">
        <div class="flex-1">
          <label for="projectName" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Nom du projet
          </label>
          <Input
            id="projectName"
            bind:value={projectName}
            placeholder="Ex: Système de gestion des commandes"
            class="text-lg"
          />
        </div>
        <div class="flex items-center gap-2 pt-6">
          <Button
            color="light"
            size="sm"
            disabled={!projectName.trim() || saving}
            onclick={saveState}
          >
            {#if saving}
              <Spinner size="4" class="mr-2" />
            {:else}
              <FloppyDiskOutline class="w-4 h-4 mr-2" />
            {/if}
            Sauvegarder
          </Button>
          {#if saveStatus}
            <span class="text-sm text-green-600 dark:text-green-400">
              {saveStatus}
            </span>
          {/if}
        </div>
      </div>
      
      {#if savedProjects.length > 0}
        <div class="flex items-center gap-4 pt-4 border-t border-gray-200 dark:border-gray-700">
          <div class="flex-1">
            <label for="loadProject" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Charger un projet existant
            </label>
            <Select
              id="loadProject"
              bind:value={selectedProject}
              placeholder="Sélectionnez un projet..."
              items={[{ value: "", name: "-- Sélectionnez --" }, ...savedProjects.map(p => ({ value: p, name: p }))]}
            />
          </div>
          <div class="pt-6">
            <Button
              color="blue"
              size="sm"
              disabled={!selectedProject || loading}
              onclick={loadState}
            >
              {#if loading}
                <Spinner size="4" class="mr-2" />
              {:else}
                <FolderOpenOutline class="w-4 h-4 mr-2" />
              {/if}
              Charger
            </Button>
          </div>
        </div>
      {/if}
    </div>
  </Card>

  <!-- Main interview content -->
  <div class="flex-1 flex gap-4 min-h-0">
  {#if fullCanvasMarkdown}
    <!-- Full canvas view after completion -->
    <div class="flex-1 bg-white dark:bg-gray-800 rounded-lg shadow-lg overflow-hidden flex flex-col">
      <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between flex-none">
        <Heading tag="h2">Canvas Domain Model Complet</Heading>
        <div class="flex items-center gap-2">
          <Button
            color="blue"
            size="sm"
            disabled={!projectName.trim() || savingCanvas}
            onclick={saveFullCanvas}
          >
            {#if savingCanvas}
              <Spinner size="4" class="mr-2" />
            {:else}
              <FloppyDiskOutline class="w-4 h-4 mr-2" />
            {/if}
            Sauvegarder Canvas
          </Button>
          {#if canvasSaveStatus}
            <span class="text-sm text-green-600 dark:text-green-400">
              {canvasSaveStatus}
            </span>
          {/if}
          <Button color="light" size="sm" onclick={() => { fullCanvasMarkdown = ""; interviewState.isComplete = false; }}>
            Retour à l'interview
          </Button>
        </div>
      </div>
      <div class="flex-1 overflow-y-auto p-6">
        <div class="prose dark:prose-invert max-w-none">
          {@html fullCanvasMarkdown.split('\n').map(line => {
            if (line.startsWith('#')) return `<h2>${line.replace(/^#+\s*/, '')}</h2>`;
            if (line.startsWith('*')) return `<p>${line}</p>`;
            if (line.startsWith('|')) return line; // Keep tables as-is
            return `<p>${line}</p>`;
          }).join('')}
        </div>
      </div>
    </div>
  {:else}
    <!-- Interview view -->
    <!-- Left sidebar: Progress and sections -->
    <div class="w-64 flex-none flex flex-col gap-4 overflow-y-auto">
    <Card class="flex-none">
      <Heading tag="h3" class="mb-4">Progression</Heading>
      <div class="w-full bg-gray-200 rounded-full h-2.5 dark:bg-gray-700">
        <div class="bg-blue-600 h-2.5 rounded-full transition-all duration-300" style="width: {progress}%"></div>
      </div>
      <p class="text-sm text-gray-600 dark:text-gray-400 mt-2">
        Section {interviewState.currentSection + 1} sur {sections.length} — {progress}%
      </p>
    </Card>

    <Card class="flex-none">
      <Heading tag="h4" class="mb-3">Sections</Heading>
      <div class="space-y-2">
        {#each sections as section, idx}
          <button
            class="w-full text-left px-3 py-2 rounded-lg transition-colors hover:bg-gray-100 dark:hover:bg-gray-700
                   {idx === interviewState.currentSection ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-400' : ''}"
            onclick={() => jumpToSection(idx)}
          >
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium">{section.title}</span>
              {#if section.completed}
                <CheckCircleSolid class="w-4 h-4 text-green-500" />
              {:else if idx === interviewState.currentSection}
                <Badge color="blue">En cours</Badge>
              {/if}
            </div>
          </button>
        {/each}
      </div>
    </Card>
  </div>

  <!-- Main content: Question and answer -->
  <div class="flex-1 flex flex-col gap-4 min-h-0 overflow-y-auto">
    {#if !interviewState.isComplete}
      <Card class="flex-none">
        <div class="mb-4">
          <Badge color="indigo" large>Question {interviewState.currentQuestionIndex + 1} / {currentSection.questions.length}</Badge>
          <Heading tag="h2" class="mt-2">{currentSection.title}</Heading>
        </div>

        <div class="space-y-4">
          <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-6">
            <p class="text-lg text-gray-900 dark:text-gray-100 leading-relaxed">
              {currentQuestion}
            </p>
          </div>

          <div>
            <label for="answer" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Votre réponse
            </label>
            <AudioInput 
              bind:value={currentAnswer} 
              onSubmit={handleAnswerSubmit}
              placeholder="Répondez ici (texte ou audio)..."
            />
          </div>
        </div>

        <div class="flex justify-between mt-6 pt-4 border-t border-gray-200 dark:border-gray-700">
          <Button
            color="light"
            disabled={!canGoPrevious}
            onclick={goToPrevious}
          >
            <ChevronLeftOutline class="w-4 h-4 mr-2" />
            Précédent
          </Button>

          <Button
            color="blue"
            disabled={!canGoNext}
            onclick={saveAndNext}
          >
            Suivant
            <ChevronRightOutline class="w-4 h-4 ml-2" />
          </Button>
        </div>
      </Card>
    {:else}
      <Card class="flex-none">
        <div class="text-center py-12">
          <CheckCircleSolid class="w-16 h-16 text-green-500 mx-auto mb-4" />
          <Heading tag="h2" class="mb-2">Interview complétée !</Heading>
          <p class="text-gray-600 dark:text-gray-400 mb-6">
            Toutes les sections ont été remplies. Le canvas de domaine peut maintenant être généré.
          </p>
          
          {#if !fullCanvasMarkdown}
            <Button 
              color="blue" 
              size="lg"
              disabled={generatingCanvas}
              onclick={handleGenerateCanvas}
            >
              {#if generatingCanvas}
                <Spinner size="4" class="mr-2" />
                Génération en cours...
              {:else}
                Générer le Domain Model Canvas
              {/if}
            </Button>
          {:else}
            <Button color="green" size="lg">
              Canvas généré avec succès !
            </Button>
          {/if}
        </div>
      </Card>
    {/if}

    {#if processing}
      <Card class="flex-none">
        <div class="flex items-center gap-3 text-blue-600 dark:text-blue-400">
          <Spinner size="6" />
          <div>
            <p class="font-medium">Traitement en cours...</p>
            <p class="text-sm">Le LLM analyse vos réponses pour remplir le canvas</p>
          </div>
        </div>
      </Card>
    {/if}

    {#if error}
      <Card class="flex-none">
        <div class="text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 p-4 rounded-lg">
          <p class="font-semibold">Erreur</p>
          <p class="text-sm mt-1">{error}</p>
        </div>
      </Card>
    {/if}

    <!-- Answers summary -->
    {#if interviewState.answers.length > 0}
      <Card class="flex-none">
        <Heading tag="h4" class="mb-3">Réponses récentes</Heading>
        <div class="space-y-2 max-h-48 overflow-y-auto">
          {#each interviewState.answers.slice(-3).reverse() as answer}
            <div class="text-sm bg-gray-50 dark:bg-gray-800 p-3 rounded border border-gray-200 dark:border-gray-700">
              <p class="font-medium text-gray-700 dark:text-gray-300 mb-1">
                {answer.question}
              </p>
              <p class="text-gray-600 dark:text-gray-400 line-clamp-2">
                {answer.answer}
              </p>
            </div>
          {/each}
        </div>
      </Card>
    {/if}
  </div>

  <!-- Right panel: Real-time canvas preview -->
  {#if processedSections.length > 0 && !interviewState.isComplete}
    <div class="w-96 flex-none flex flex-col min-h-0">
      <Card class="flex-1 flex flex-col min-h-0">
        <div class="flex items-center justify-between mb-4">
          <Heading tag="h3">Aperçu du Canvas</Heading>
          <Button
            color="light"
            size="xs"
            disabled={!projectName.trim() || savingCanvas}
            onclick={saveCanvasPreview}
          >
            {#if savingCanvas}
              <Spinner size="3" class="mr-1" />
            {:else}
              <FloppyDiskOutline class="w-3 h-3 mr-1" />
            {/if}
            Sauvegarder
          </Button>
        </div>
        {#if canvasSaveStatus}
          <div class="mb-3 text-xs text-green-600 dark:text-green-400">
            {canvasSaveStatus}
          </div>
        {/if}
        <div class="flex-1 overflow-y-auto space-y-4 text-sm">
          {#each processedSections as section}
            <div class="border-l-4 border-blue-500 pl-3">
              <p class="font-semibold text-gray-900 dark:text-gray-100">
                {section.section_title}
              </p>
              <div class="mt-2 text-gray-600 dark:text-gray-400 line-clamp-4">
                {section.canvas_content.substring(0, 200)}...
              </div>
            </div>
          {/each}
        </div>
        <div class="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
          <p class="text-xs text-gray-500 dark:text-gray-400">
            {processedSections.length} / {sections.length} sections traitées
          </p>
        </div>
      </Card>
    </div>
  {/if}
  {/if}
  </div>
</div>
