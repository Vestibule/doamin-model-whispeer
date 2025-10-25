# Canvas — Rich Domain Model (DDD)

> Objectif : cadrer un domaine avec un modèle riche (entités porteuses de logique, invariants explicites, langage ubiquiste). Remplis court et concret.

---

## 1) Contexte & Vision

* **Problème à résoudre :** *…*
* **Valeur métier attendue :** *…*
* **Impacts mesurables (KPIs) :** *…*
* **Périmètre (Bounded Context) :** *…*

## 2) Langage Ubiquiste (Glossaire)

| Terme | Définition métier | Exemple |
| ----- | ----------------- | ------- |
| *…*   | *…*               | *…*     |

## 3) Acteurs & Cas d’usage critiques

* **Acteurs :** *…*
* **Top 5 use cases (orientés résultat) :**

  1. *En tant que… je veux… afin de…*
  2. *…*

## 4) Agrégats (racines, frontières, invariants)

Pour chaque agrégat : nom, racine, règles d’encapsulation, invariants transactionnels.

| Agrégat    | Racine       | Principales entités internes | Invariants (ACID dans l’agrégat)                   | Politiques (domain rules) |
| ---------- | ------------ | ---------------------------- | -------------------------------------------------- | ------------------------- |
| *Commande* | *CommandeId* | *LigneCommande, Remise*      | *Total ≥ 0; État ∈ {Brouillon, Validée, Expédiée}* | *Remise max 20%*          |

## 5) Entités & Value Objects

* **Entités (identité stable) :** *…*
* **Value Objects (immutables, égalité structurelle) :** *…*
* **Raisons de VO (précision, validation locale, invariants) :** *…*

## 6) Domain Services (si logique sans porteur naturel)

* **Nom :** *CalculPrixService*
* **Contrat :** *Prix calculer(Commande, Catalogue)*
* **Règles :** *…*

## 7) Domain Events (faits passés, nommés au parfait)

| Événement         | Quand             | Payload minimal               | Consommateurs            | Outbox ? |
| ----------------- | ----------------- | ----------------------------- | ------------------------ | -------- |
| *CommandeValidée* | *à la validation* | *commandeId, total, clientId* | *Facturation, Analytics* | *Oui*    |

## 8) Politiques, Invariants & Règles de validation

* **Invariants d’agrégat :** *…*
* **Politiques temporelles (ex : délais) :** *…*
* **Règles de calculs (arrondis, TVAs) :** *…*

## 9) Repositories & Factories (interfaces domaine)

| Type       | Port                 | Contrat minimal              | Notes                         |
| ---------- | -------------------- | ---------------------------- | ----------------------------- |
| Repository | `CommandeRepository` | `save(commande)`, `byId(id)` | *Transaction à l’agrégat*     |
| Factory    | `CommandeFactory`    | `nouvelleCommande(clientId)` | *Enforce invariants initiaux* |

## 10) Cartographie des Bounded Contexts & Relations

* **Contexts :** *Vente, Stock, Facturation…*
* **Relations :** *Conformist, Anti-Corruption Layer (ACL), Published Language*
* **Flux d’événements inter-contextes :** *…*

## 11) Intégration & Adaptateurs (Hexagonal)

* **Ports sortants (domain → infra) :** *Clock, PaymentGateway, EmailSender*
* **Adaptateurs :** *StripeAdapter, PostgresRepository, SMTPMailer*
* **Stratégie de résilience :** *Retries, Circuit Breaker, Idempotence Keys*

## 12) Modélisation d’état & transitions

Ajouter un diagramme d’états si utile.

| Agrégat    | États                                    | Transitions autorisées     | Gardiens (conditions)          |
| ---------- | ---------------------------------------- | -------------------------- | ------------------------------ |
| *Commande* | *Brouillon → Validée → Expédiée → Close* | *Valider, Expédier, Clore* | *Stock suffisant, Paiement OK* |

## 13) Sécurité & Conformité

* **Rôles & permissions au niveau domaine :** *…*
* **Traçabilité (audit log, WHO/WHEN/WHAT) :** *…*
* **Données sensibles (PII) & rétention :** *…*

## 14) Performance & Scalabilité

* **Granularité d’agrégat (éviter les “God Aggregates”) :** *…*
* **Concurrence (optimistic locking, versioning) :** *…*
* **Lecture (CQRS, projections) :** *…*

## 15) Stratégie de persistance

* **Mapping :** *VO comme colonnes, Entités enfants en tables dédiées*
* **Transactions :** *par agrégat*
* **Outbox & Message Relay :** *…*

## 16) Tests de domaine (Given-When-Then)

* **Given (state setup par builders/fixtures VO)**
* **When (commande métier)**
* **Then (état/invariants/event émis)**

### Exemple GWT (pseudo-code)

```
Étant donné une commande brouillon avec 2 lignes
Quand j’applique une remise de 10%
Alors le total est recalculé et l’invariant Total ≥ 0 est respecté
Et l’événement RemiseAppliquée est émis
```

## 17) Observabilité métier

* **Événements domaine instrumentés :** *…*
* **KPI dérivés (projections) :** *…*

## 18) Risques & pièges

* **Anémie du modèle (logique dans services applicatifs)**
* **Agrégats trop gros (verrous, contention)**
* **Événements “données” plutôt que “faits métier”**

## 19) Décisions d’architecture (ADR)

| Décision | Contexte | Option choisie | Conséquence |
| -------- | -------- | -------------- | ----------- |
| *…*      | *…*      | *…*            | *…*         |

## 20) Roadmap Domain-first

1. **MVP agrégat principal** (invariants, events, repo)
2. **Ports/Adaptateurs critiques** (paiement, email)
3. **Projections lecture & ACL**
4. **Tests scénarisés & KPI**

---

### Checklists éclair

*

> Astuce : si une règle traverse plusieurs agrégats, reconsidère la frontière ou déplace en **Policy** orchestrée par events.
