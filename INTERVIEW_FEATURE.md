# Interview Guidée - Feature Documentation

## Vue d'ensemble

Cette fonctionnalité implémente un mode d'interview guidée pour remplir le Domain Model Canvas de manière structurée, basée sur les questions définies dans `samples/domain-model-questions.md`.

## Architecture

### Composants créés

#### 1. **Types TypeScript** (`src/lib/types/interview.ts`)
Définit la structure de données pour :
- `InterviewSection` : Une section de l'interview avec ses questions
- `UserAnswer` : Une réponse de l'utilisateur avec timestamp
- `CanvasContent` : La structure complète du canvas de domaine
- `InterviewState` : L'état actuel de l'interview
- `INTERVIEW_SECTIONS` : Les 9 sections principales de l'interview

#### 2. **DomainModelInterview.svelte** (`src/lib/DomainModelInterview.svelte`)
Composant principal de l'interview qui :
- Affiche les questions une par une
- Utilise le composant AudioInput pour la saisie (texte ou audio)
- Suit la progression à travers les 9 sections
- Permet la navigation entre questions et sections
- Stocke les réponses dans l'état local
- Affiche un résumé des réponses récentes

**Features principales :**
- Barre de progression visuelle
- Navigation avant/arrière entre questions
- Accès rapide aux sections via sidebar
- Marquage des sections complétées
- Support audio et texte pour les réponses

#### 3. **CanvasViewer.svelte** (`src/lib/CanvasViewer.svelte`)
Composant pour visualiser le canvas de domaine en cours de construction :
- Affiche les différentes sections du canvas
- Utilise des tables Flowbite pour les données structurées
- S'adapte dynamiquement au contenu disponible
- Support du mode sombre

#### 4. **Intégration dans App.svelte**
- Ajout d'un toggle pour basculer entre "Interview guidée" et "Transcript libre"
- Mode interview par défaut
- Icons : MessageDotsOutline pour interview, FileCodeOutline pour transcript

## Les 9 Sections de l'Interview

1. **Contexte & Vision** - Comprendre le problème et la valeur métier
2. **Acteurs & Use Cases** - Identifier les utilisateurs et actions clés
3. **Langage Ubiquiste** - Définir le vocabulaire métier
4. **Agrégats & Entités/Value Objects** - Modéliser les objets du domaine
5. **Domain Events & Règles** - Capturer les événements métier
6. **Contextes & Intégration** - Architecture hexagonale
7. **Sécurité, Performance, Persistance** - Contraintes techniques
8. **Tests de domaine & KPI** - Validation et mesures
9. **Roadmap Domain-first** - Planification de la livraison

## État actuel

### ✅ Implémenté

#### Frontend (TypeScript + Svelte 5)
- Structure de types TypeScript complète
- Interface utilisateur de l'interview
- Navigation fluide entre questions
- Saisie audio et texte
- **Visualisation temps réel du canvas** pendant l'interview
- Basculement entre modes
- Affichage markdown du canvas complet
- Gestion des erreurs et états de chargement
- Build successful

#### Backend (Rust + Tauri)
- ✅ Module `interview.rs` avec types Serde
- ✅ `InterviewProcessor` qui utilise le LLM pour transformer les réponses
- ✅ Commande `process_interview_section` - traite une section complète avec le LLM
- ✅ Commande `generate_full_canvas` - compile toutes les sections en markdown
- ✅ Prompts système spécifiques par section pour guider le LLM
- ✅ Support Ollama et LLM externes via `llm_router`
- ✅ Nouvelle méthode `generate_text()` dans `LlmRouter`
- ✅ Tests unitaires
- ✅ Compilation réussie (cargo check)

#### Intégration
- ✅ Bindings TypeScript pour les commandes Tauri
- ✅ Appel automatique du LLM à la fin de chaque section
- ✅ Aperçu temps réel des sections traitées
- ✅ Génération finale du canvas complet
- ✅ Vue split : interview + preview du canvas

### 🔄 Prochaines améliorations possibles

1. **Sauvegarde/chargement d'interviews**
   - Persister l'état de l'interview localement
   - Reprendre une interview en cours

2. **Validation des réponses par le LLM**
   - Le LLM peut suggérer si la réponse manque de spécificité
   - Demander des exemples concrets si nécessaire

3. **Export & partage**
   - Export en PDF avec mise en page professionnelle
   - Copier le markdown dans le presse-papier
   - Sauvegarder en fichier .md

## Utilisation

```bash
# Démarrer l'application
pnpm dev
# ou
task dev

# L'application démarre en mode "Interview guidée" par défaut
# Cliquer sur le toggle pour basculer entre modes
```

### Flux utilisateur

1. L'utilisateur démarre l'interview
2. Pour chaque question :
   - Lire la question affichée
   - Répondre via texte ou audio
   - Cliquer "Suivant" pour passer à la question suivante
3. Navigation libre entre sections via la sidebar
4. À la fin, générer le canvas complet

## Prochaines étapes recommandées

1. **Implémenter les commandes Tauri backend** (voir TODO restant)
   - Intégrer avec l'orchestration LLM existante
   - Adapter pour le format question-réponse

2. **Améliorer le CanvasViewer**
   - Permettre l'édition inline des sections
   - Export en markdown/PDF

3. **Persistence**
   - Sauvegarder l'état de l'interview localement
   - Reprendre une session interrompue

4. **Validation des réponses**
   - Le LLM peut valider si la réponse est suffisamment concrète
   - Demander des exemples si la réponse est trop vague

5. **Vue côte-à-côte**
   - Afficher l'interview et le canvas en même temps
   - Mise à jour temps réel du canvas pendant l'interview

## Philosophie

Cette fonctionnalité suit les principes du guide opératoire :
- **Questions directes** : Pas d'édulcoration, focus sur la valeur
- **Validation continue** : Reformuler et confirmer avant d'écrire
- **Spécificité** : Demander des exemples concrets
- **Progressif** : Une section à la fois, ne pas surcharger
- **Orienté invariants** : Toujours chercher les règles métier fondamentales

## Références

- Guide des questions : `samples/domain-model-questions.md`
- Template du canvas : `samples/domain-model-canvas.md`
- WARP rules : `WARP.md`
