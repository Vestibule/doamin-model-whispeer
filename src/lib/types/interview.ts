export interface InterviewSection {
  id: number;
  title: string;
  questions: string[];
  canvasSection: string;
  completed: boolean;
}

export interface UserAnswer {
  sectionId: number;
  questionIndex: number;
  question: string;
  answer: string;
  timestamp: Date;
}

export interface CanvasContent {
  context: string;
  ubiquitousLanguage: Array<{ term: string; definition: string; example: string }>;
  actors: string;
  useCases: string[];
  aggregates: Array<{
    name: string;
    root: string;
    entities: string;
    invariants: string;
    policies: string;
  }>;
  entities: string;
  valueObjects: string;
  domainServices: Array<{ name: string; contract: string; rules: string }>;
  domainEvents: Array<{
    event: string;
    when: string;
    payload: string;
    consumers: string;
    outbox: boolean;
  }>;
  policies: string;
  repositories: Array<{ type: string; port: string; contract: string; notes: string }>;
  boundedContexts: string;
  integration: string;
  stateModeling: string;
  security: string;
  performance: string;
  persistence: string;
  tests: string;
  observability: string;
  risks: string;
  adr: Array<{ decision: string; context: string; option: string; consequence: string }>;
  roadmap: string[];
}

export interface InterviewState {
  currentSection: number;
  currentQuestionIndex: number;
  answers: UserAnswer[];
  canvas: Partial<CanvasContent>;
  isComplete: boolean;
}

// The 9 main sections from the questions document
export const INTERVIEW_SECTIONS: InterviewSection[] = [
  {
    id: 1,
    title: "Contexte & Vision",
    questions: [
      "Quel problème réel veux-tu résoudre ? Qui en souffre aujourd'hui ?",
      "Quelle transformation visible pour l'utilisateur final ?",
      "Comment mesurer le succès ? (KPIs pratiques)",
      "Quel est le périmètre strict du premier contexte ?"
    ],
    canvasSection: "context",
    completed: false
  },
  {
    id: 2,
    title: "Acteurs & Use Cases",
    questions: [
      "Qui utilise le système ? (Profils concrets)",
      "Quelles actions gagnantes doivent absolument réussir ?",
      "Quelles actions génèrent la valeur principale ?"
    ],
    canvasSection: "actors-usecases",
    completed: false
  },
  {
    id: 3,
    title: "Langage Ubiquiste",
    questions: [
      "Quels mots doivent avoir une seule définition ici ?",
      "Quel est l'exemple concret associé à chaque terme ?"
    ],
    canvasSection: "ubiquitousLanguage",
    completed: false
  },
  {
    id: 4,
    title: "Agrégats & Entités/Value Objects",
    questions: [
      "Quels objets doivent évoluer ensemble ou échouer ensemble ?",
      "Quelle règle métier ne doit jamais être cassée ? (invariant)",
      "Qu'est-ce qui possède quoi ? (ownership)",
      "Qu'est-ce qui a une identité stable → Entité ?",
      "Qu'est-ce qui est une valeur → VO ?"
    ],
    canvasSection: "aggregates-entities",
    completed: false
  },
  {
    id: 5,
    title: "Domain Events & Règles",
    questions: [
      "Quel événement métier marque un changement d'état important ?",
      "Qui doit réagir ? Interne ou externe au domaine ?"
    ],
    canvasSection: "events-rules",
    completed: false
  },
  {
    id: 6,
    title: "Contextes & Intégration (Hexagonal)",
    questions: [
      "Qui fournit des données externes au domaine ? (ports entrants)",
      "Qui consomme des actions du domaine ? (ports sortants)",
      "Y a-t-il d'autres contextes métier ? Comment partager ou protéger le langage ?"
    ],
    canvasSection: "contexts-integration",
    completed: false
  },
  {
    id: 7,
    title: "Sécurité, Performance, Persistance",
    questions: [
      "Y a-t-il des données sensibles ? Qui y accède ?",
      "Risque de charge élevée ou de concurrence ?",
      "Stratégie de persistance (simple au début)"
    ],
    canvasSection: "security-performance",
    completed: false
  },
  {
    id: 8,
    title: "Tests de domaine & KPI",
    questions: [
      "Quel scénario critique doit toujours fonctionner ?",
      "Quels KPI démontrent la réussite métier ?"
    ],
    canvasSection: "tests-kpi",
    completed: false
  },
  {
    id: 9,
    title: "Roadmap Domain-first",
    questions: [
      "Quel est le plus petit système utile complet ?",
      "Dans quel ordre les agrégats émergent ?"
    ],
    canvasSection: "roadmap",
    completed: false
  }
];
