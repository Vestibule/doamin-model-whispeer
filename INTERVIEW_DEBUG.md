# Interview Processing Debug Guide

## Symptôme
L'interface affiche "Traitement en cours... Le LLM analyse vos réponses pour remplir le canvas" mais rien ne se passe.

## Causes possibles

### 1. Ollama n'est pas démarré
**Solution :**
```bash
# Vérifier si Ollama est actif
curl http://localhost:11434/api/tags

# Si erreur, démarrer Ollama
ollama serve
```

### 2. Le modèle n'existe pas
**Vérifier :**
```bash
ollama list
```

**Solution :** Le code utilise le modèle spécifié dans `OLLAMA_MODEL` (défaut: `domain-model-mistral`).

Si le modèle n'existe pas :
```bash
# Option 1: Utiliser un modèle existant
export OLLAMA_MODEL=llama2  # ou mistral, gemma, etc.

# Option 2: Télécharger le modèle
ollama pull domain-model-mistral  # ou le modèle de votre choix
```

### 3. Vérifier les logs
Les nouveaux logs ajoutés permettent de voir exactement où ça bloque :

```bash
# Relancer l'app avec les logs visibles
task dev

# Chercher dans les logs:
# - [Interview] Starting to process section
# - [LLM Router] Generating text with Ollama
# - [LLM Router] Sending POST request to Ollama...
# - [LLM Router] Received response with status
```

### 4. Timeout
Si le LLM prend trop de temps (> 2 minutes), un timeout se déclenchera avec le message :
"LLM request timed out after 120 seconds"

## Configuration recommandée

Créer/modifier `.env` à la racine du projet :

```env
# Utiliser Ollama local (recommandé)
LLM_PROVIDER=ollama
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=llama2  # ou mistral, gemma, etc.
```

## Test rapide

1. Vérifier Ollama :
```bash
curl http://localhost:11434/api/tags
```

2. Tester une génération :
```bash
curl http://localhost:11434/api/generate -d '{
  "model": "llama2",
  "prompt": "Hello world",
  "stream": false
}'
```

3. Si ça fonctionne, relancer l'app :
```bash
task dev
```

## Notes
- Le timeout a été ajouté à 120 secondes (2 minutes)
- Des logs détaillés ont été ajoutés pour tracer chaque étape
- L'erreur sera maintenant visible dans l'UI si le LLM échoue
