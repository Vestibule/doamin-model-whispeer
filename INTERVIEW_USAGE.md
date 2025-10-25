# Guide d'utilisation - Interview Domain Model

## Vue d'ensemble

Cette application vous guide à travers une interview structurée en 9 sections pour construire un Canvas Domain Model complet. Le LLM analyse vos réponses en temps réel et génère automatiquement la documentation DDD.

## Prérequis

### Configuration LLM

L'application nécessite un LLM configuré. Deux options :

#### Option 1: LLM Externe (OpenAI-compatible)
Créez un fichier `.env` à la racine du projet :

```env
LLM_PROVIDER=external
LLM_API_KEY=votre_clé_api
LLM_ENDPOINT=https://api.openai.com/v1/chat/completions
```

#### Option 2: Ollama (Local)
```env
LLM_PROVIDER=ollama
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=domain-model-mistral
```

Assurez-vous qu'Ollama est lancé avec le modèle approprié :
```bash
ollama pull mistral
ollama serve
```

## Démarrage

```bash
# Mode développement (avec hot reload)
pnpm dev
# ou
task dev

# L'application s'ouvre dans une nouvelle fenêtre
# Par défaut, vous êtes en mode "Interview guidée"
```

## Processus d'interview

### Étape 1: Les 9 sections

L'interview est divisée en 9 sections thématiques :

1. **Contexte & Vision** (4 questions)
   - Problème à résoudre
   - Transformation pour l'utilisateur
   - KPIs de succès
   - Périmètre du contexte

2. **Acteurs & Use Cases** (3 questions)
   - Profils utilisateurs
   - Actions gagnantes
   - Valeur principale

3. **Langage Ubiquiste** (2 questions)
   - Termes métier
   - Exemples concrets

4. **Agrégats & Entités/Value Objects** (5 questions)
   - Objets qui évoluent ensemble
   - Invariants
   - Ownership
   - Entités vs Value Objects

5. **Domain Events & Règles** (2 questions)
   - Événements métier
   - Réactions (internes/externes)

6. **Contextes & Intégration** (3 questions)
   - Ports entrants
   - Ports sortants
   - Autres bounded contexts

7. **Sécurité, Performance, Persistance** (3 questions)
   - Données sensibles
   - Charge et concurrence
   - Stratégie de persistance

8. **Tests de domaine & KPI** (2 questions)
   - Scénarios critiques
   - KPI métier

9. **Roadmap Domain-first** (2 questions)
   - MVP minimal
   - Ordre d'émergence des agrégats

### Étape 2: Répondre aux questions

Pour chaque question :

1. **Lisez attentivement** la question affichée
2. **Répondez de manière concrète** - évitez le jargon vague
3. Utilisez soit :
   - **Le champ texte** pour saisir directement
   - **Le bouton micro** pour dicter votre réponse (reconnaissance vocale)
4. **Cliquez "Suivant"** pour passer à la question suivante

**💡 Conseils pour de bonnes réponses :**
- Donnez des **exemples concrets** plutôt que des généralités
- Pensez **valeur métier** plutôt que technique
- Soyez **spécifique** sur les invariants et règles
- N'hésitez pas à mentionner des cas limites

### Étape 3: Traitement LLM automatique

À la **fin de chaque section** (après la dernière question) :
- ✨ Le LLM traite automatiquement vos réponses
- 📝 Il reformule en langage métier clair
- 📊 Il structure selon le format Canvas
- ⚡ Cela prend environ 5-15 secondes

Vous voyez apparaître :
```
Traitement en cours...
Le LLM analyse vos réponses pour remplir le canvas
```

### Étape 4: Aperçu temps réel

Pendant que vous progressez, un **panneau latéral droit** apparaît avec :
- Les sections déjà traitées
- Un aperçu du contenu généré
- Le nombre de sections complétées (ex: "3 / 9 sections traitées")

### Étape 5: Navigation libre

Vous pouvez à tout moment :
- **Revenir en arrière** avec le bouton "Précédent"
- **Sauter à une section** en cliquant dans la barre latérale gauche
- Les sections complétées sont marquées d'une ✓ verte

### Étape 6: Génération finale

Après avoir répondu à toutes les questions :

1. Un message s'affiche : **"Interview complétée !"**
2. Cliquez sur **"Générer le Domain Model Canvas"**
3. Le LLM compile toutes les sections (5-10 secondes)
4. Le **Canvas complet** s'affiche en markdown formaté

Le canvas contient :
- Toutes les 9 sections remplies
- Format markdown professionnel
- Tables, listes, et structure claire
- Prêt à être copié ou exporté

## Fonctionnalités avancées

### Basculer entre modes

En haut à droite, deux boutons permettent de basculer :
- **Interview guidée** : Le processus structuré décrit ici
- **Transcript libre** : Mode original (soumission libre de transcript)

### Retour à l'interview

Depuis la vue du canvas final, cliquez sur **"Retour à l'interview"** pour :
- Revoir vos réponses
- Modifier des sections
- Régénérer le canvas

## Architecture technique

### Frontend
- **Svelte 5** avec runes ($state, $derived)
- **Flowbite components** pour l'UI
- **Tailwind CSS** pour le style
- Types TypeScript stricts

### Backend
- **Rust + Tauri** pour la sécurité et performance
- **LLM Router** : abstraction multi-provider (Ollama, OpenAI, etc.)
- **Interview Processor** : logique métier de transformation
- Prompts système spécialisés par section

### Flux de données
```
User Input (audio/text)
    ↓
Frontend (Svelte)
    ↓
Tauri Commands (process_interview_section)
    ↓
InterviewProcessor (Rust)
    ↓
LlmRouter → LLM API
    ↓
Canvas Content (markdown)
    ↓
Frontend Display
```

## Dépannage

### Le LLM ne répond pas
- Vérifiez votre fichier `.env`
- Pour Ollama : `ollama list` pour voir les modèles disponibles
- Vérifiez les logs : l'app affiche les erreurs dans l'interface

### Réponses de mauvaise qualité
- Soyez plus spécifique dans vos réponses
- Donnez des exemples concrets
- Évitez le jargon technique prématuré

### L'application se bloque
- Vérifiez la console pour les erreurs
- Rechargez l'application
- Vos réponses ne sont pas encore sauvegardées (feature à venir)

## Exemples de réponses

### ❌ Mauvaise réponse (trop vague)
**Q: Quel problème réel veux-tu résoudre ?**
> "Améliorer la gestion des données"

### ✅ Bonne réponse (concrète)
**Q: Quel problème réel veux-tu résoudre ?**
> "Les vendeurs perdent 2h/jour à chercher si un produit est en stock avant de valider une commande. Cela entraîne 15% de commandes annulées et des clients mécontents."

### ❌ Mauvaise réponse (jargon prématuré)
**Q: Quels objets doivent évoluer ensemble ?**
> "Un pattern repository avec des entities agrégées par foreign keys"

### ✅ Bonne réponse (langage métier)
**Q: Quels objets doivent évoluer ensemble ?**
> "Une Commande avec ses LignesCommande et sa Remise. Si on annule la commande, toutes les lignes doivent être annulées en même temps. Le total doit toujours être cohérent avec les lignes."

## Références

- **Guide des questions** : `samples/domain-model-questions.md`
- **Template du canvas** : `samples/domain-model-canvas.md`
- **Documentation technique** : `INTERVIEW_FEATURE.md`
- **Domain-Driven Design** : Livre d'Eric Evans

## Support

Pour des questions ou suggestions :
- Ouvrir une issue sur le repo
- Consulter la documentation DDD
- Vérifier les logs dans la console développeur
