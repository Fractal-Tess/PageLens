<script lang="ts">
  import { Card, CardContent, CardHeader, CardTitle } from '../../ui/card';
  import { Checkbox } from '../../ui/checkbox';
  import { Label } from '../../ui/label';
  import type { Snippet } from 'svelte';
  
  interface Option {
    id: string;
    label: string;
    checked: boolean;
    disabled?: boolean;
  }
  
  interface Props {
    title: string;
    options: Option[];
    onChange: (id: string, checked: boolean) => void;
    children?: Snippet;
    class?: string;
  }
  
  let { title, options, onChange, children, class: className = '' }: Props = $props();
</script>

<Card class={className}>
  <CardHeader>
    <CardTitle class="text-sm font-medium">{title}</CardTitle>
  </CardHeader>
  <CardContent class="space-y-3">
    <div class="grid gap-3 sm:grid-cols-2">
      {#each options as option}
        <div class="flex items-center space-x-2">
          <Checkbox
            id={option.id}
            checked={option.checked}
            onCheckedChange={(v) => onChange(option.id, Boolean(v))}
            disabled={option.disabled}
          />
          <Label for={option.id} class="text-sm font-normal">{option.label}</Label>
        </div>
      {/each}
    </div>
    {@render children?.()}
  </CardContent>
</Card>
