<script lang="ts">
  import { Card, Heading, Badge, Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from 'flowbite-svelte';
  import type { CanvasContent } from './types/interview';

  interface Props {
    canvas: Partial<CanvasContent>;
  }

  let { canvas }: Props = $props();
</script>

<div class="h-full overflow-y-auto p-6 bg-white dark:bg-gray-800 space-y-6">
  <div class="max-w-5xl mx-auto">
    <div class="mb-6">
      <Heading tag="h1" class="mb-2">Canvas — Rich Domain Model (DDD)</Heading>
      <p class="text-gray-600 dark:text-gray-400">
        Modèle de domaine en cours de construction
      </p>
    </div>

    <!-- 1. Contexte & Vision -->
    {#if canvas.context}
      <Card>
        <Heading tag="h2" class="mb-4">1) Contexte & Vision</Heading>
        <div class="prose dark:prose-invert max-w-none">
          <p class="whitespace-pre-wrap">{canvas.context}</p>
        </div>
      </Card>
    {/if}

    <!-- 2. Langage Ubiquiste -->
    {#if canvas.ubiquitousLanguage && canvas.ubiquitousLanguage.length > 0}
      <Card>
        <Heading tag="h2" class="mb-4">2) Langage Ubiquiste (Glossaire)</Heading>
        <Table>
          <TableHead>
            <TableHeadCell>Terme</TableHeadCell>
            <TableHeadCell>Définition métier</TableHeadCell>
            <TableHeadCell>Exemple</TableHeadCell>
          </TableHead>
          <TableBody>
            {#each canvas.ubiquitousLanguage as term}
              <TableBodyRow>
                <TableBodyCell><strong>{term.term}</strong></TableBodyCell>
                <TableBodyCell>{term.definition}</TableBodyCell>
                <TableBodyCell class="text-gray-600 dark:text-gray-400">{term.example}</TableBodyCell>
              </TableBodyRow>
            {/each}
          </TableBody>
        </Table>
      </Card>
    {/if}

    <!-- 3. Acteurs & Use Cases -->
    {#if canvas.actors || (canvas.useCases && canvas.useCases.length > 0)}
      <Card>
        <Heading tag="h2" class="mb-4">3) Acteurs & Cas d'usage critiques</Heading>
        {#if canvas.actors}
          <div class="mb-4">
            <strong class="text-gray-700 dark:text-gray-300">Acteurs:</strong>
            <p class="mt-1">{canvas.actors}</p>
          </div>
        {/if}
        {#if canvas.useCases && canvas.useCases.length > 0}
          <div>
            <strong class="text-gray-700 dark:text-gray-300">Top use cases:</strong>
            <ol class="list-decimal list-inside mt-2 space-y-1">
              {#each canvas.useCases as useCase}
                <li class="text-gray-800 dark:text-gray-200">{useCase}</li>
              {/each}
            </ol>
          </div>
        {/if}
      </Card>
    {/if}

    <!-- 4. Agrégats -->
    {#if canvas.aggregates && canvas.aggregates.length > 0}
      <Card>
        <Heading tag="h2" class="mb-4">4) Agrégats (racines, frontières, invariants)</Heading>
        <Table>
          <TableHead>
            <TableHeadCell>Agrégat</TableHeadCell>
            <TableHeadCell>Racine</TableHeadCell>
            <TableHeadCell>Entités internes</TableHeadCell>
            <TableHeadCell>Invariants</TableHeadCell>
            <TableHeadCell>Politiques</TableHeadCell>
          </TableHead>
          <TableBody>
            {#each canvas.aggregates as aggregate}
              <TableBodyRow>
                <TableBodyCell><strong>{aggregate.name}</strong></TableBodyCell>
                <TableBodyCell>{aggregate.root}</TableBodyCell>
                <TableBodyCell>{aggregate.entities}</TableBodyCell>
                <TableBodyCell class="text-sm">{aggregate.invariants}</TableBodyCell>
                <TableBodyCell class="text-sm">{aggregate.policies}</TableBodyCell>
              </TableBodyRow>
            {/each}
          </TableBody>
        </Table>
      </Card>
    {/if}

    <!-- 5. Entités & Value Objects -->
    {#if canvas.entities || canvas.valueObjects}
      <Card>
        <Heading tag="h2" class="mb-4">5) Entités & Value Objects</Heading>
        {#if canvas.entities}
          <div class="mb-4">
            <strong class="text-gray-700 dark:text-gray-300">Entités (identité stable):</strong>
            <p class="mt-1 whitespace-pre-wrap">{canvas.entities}</p>
          </div>
        {/if}
        {#if canvas.valueObjects}
          <div>
            <strong class="text-gray-700 dark:text-gray-300">Value Objects:</strong>
            <p class="mt-1 whitespace-pre-wrap">{canvas.valueObjects}</p>
          </div>
        {/if}
      </Card>
    {/if}

    <!-- 7. Domain Events -->
    {#if canvas.domainEvents && canvas.domainEvents.length > 0}
      <Card>
        <Heading tag="h2" class="mb-4">7) Domain Events (faits passés)</Heading>
        <Table>
          <TableHead>
            <TableHeadCell>Événement</TableHeadCell>
            <TableHeadCell>Quand</TableHeadCell>
            <TableHeadCell>Payload minimal</TableHeadCell>
            <TableHeadCell>Consommateurs</TableHeadCell>
            <TableHeadCell>Outbox</TableHeadCell>
          </TableHead>
          <TableBody>
            {#each canvas.domainEvents as event}
              <TableBodyRow>
                <TableBodyCell><strong>{event.event}</strong></TableBodyCell>
                <TableBodyCell>{event.when}</TableBodyCell>
                <TableBodyCell class="text-sm">{event.payload}</TableBodyCell>
                <TableBodyCell class="text-sm">{event.consumers}</TableBodyCell>
                <TableBodyCell>
                  <Badge color={event.outbox ? "green" : "gray"}>
                    {event.outbox ? "Oui" : "Non"}
                  </Badge>
                </TableBodyCell>
              </TableBodyRow>
            {/each}
          </TableBody>
        </Table>
      </Card>
    {/if}

    <!-- Other sections as text blocks -->
    {#if canvas.policies}
      <Card>
        <Heading tag="h2" class="mb-4">8) Politiques & Invariants</Heading>
        <div class="prose dark:prose-invert max-w-none">
          <p class="whitespace-pre-wrap">{canvas.policies}</p>
        </div>
      </Card>
    {/if}

    {#if canvas.boundedContexts}
      <Card>
        <Heading tag="h2" class="mb-4">10) Bounded Contexts & Relations</Heading>
        <div class="prose dark:prose-invert max-w-none">
          <p class="whitespace-pre-wrap">{canvas.boundedContexts}</p>
        </div>
      </Card>
    {/if}

    {#if canvas.integration}
      <Card>
        <Heading tag="h2" class="mb-4">11) Intégration & Adaptateurs (Hexagonal)</Heading>
        <div class="prose dark:prose-invert max-w-none">
          <p class="whitespace-pre-wrap">{canvas.integration}</p>
        </div>
      </Card>
    {/if}

    {#if canvas.security}
      <Card>
        <Heading tag="h2" class="mb-4">13) Sécurité & Conformité</Heading>
        <div class="prose dark:prose-invert max-w-none">
          <p class="whitespace-pre-wrap">{canvas.security}</p>
        </div>
      </Card>
    {/if}

    {#if canvas.performance}
      <Card>
        <Heading tag="h2" class="mb-4">14) Performance & Scalabilité</Heading>
        <div class="prose dark:prose-invert max-w-none">
          <p class="whitespace-pre-wrap">{canvas.performance}</p>
        </div>
      </Card>
    {/if}

    {#if canvas.tests}
      <Card>
        <Heading tag="h2" class="mb-4">16) Tests de domaine</Heading>
        <div class="prose dark:prose-invert max-w-none">
          <p class="whitespace-pre-wrap">{canvas.tests}</p>
        </div>
      </Card>
    {/if}

    {#if canvas.roadmap && canvas.roadmap.length > 0}
      <Card>
        <Heading tag="h2" class="mb-4">20) Roadmap Domain-first</Heading>
        <ol class="list-decimal list-inside space-y-2">
          {#each canvas.roadmap as step}
            <li class="text-gray-800 dark:text-gray-200">{step}</li>
          {/each}
        </ol>
      </Card>
    {/if}

    {#if Object.keys(canvas).length === 0}
      <Card>
        <div class="text-center py-12 text-gray-500 dark:text-gray-400">
          <p class="text-lg mb-2">Le canvas est vide</p>
          <p class="text-sm">Répondez aux questions de l'interview pour commencer à le remplir.</p>
        </div>
      </Card>
    {/if}
  </div>
</div>
