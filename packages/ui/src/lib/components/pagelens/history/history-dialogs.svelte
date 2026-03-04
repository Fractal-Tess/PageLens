<script lang="ts">
  import { Button } from '../../ui/button'
  import { Input } from '../../ui/input'
  import { Label } from '../../ui/label'
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
    DialogFooter,
    DialogClose
  } from '../../ui/dialog'
  import { Trash2, Download, Upload, TriangleAlert } from '@lucide/svelte'
  import type {
    HistoryDialogState,
    HistoryState
  } from '../../../stores/history.svelte'

  interface Props {
    historyState: HistoryState
    dialogState: HistoryDialogState
    onCloseRename: () => void
    onCloseDelete: () => void
    onCloseDeleteAll: () => void
    onCloseExportRun: () => void
    onCloseExport: () => void
    onCloseImport: () => void
    onRenameInput: (value: string) => void
    onExportRunInput: (value: string) => void
    onExportInput: (value: string) => void
    onImportInput: (value: string) => void
    onConfirmRename: () => void
    onConfirmDelete: () => void
    onConfirmDeleteAll: () => void
    onConfirmExportRun: () => void
    onConfirmExport: () => void
    onConfirmImport: () => void
  }

  let {
    historyState,
    dialogState,
    onCloseRename,
    onCloseDelete,
    onCloseDeleteAll,
    onCloseExportRun,
    onCloseExport,
    onCloseImport,
    onRenameInput,
    onExportRunInput,
    onExportInput,
    onImportInput,
    onConfirmRename,
    onConfirmDelete,
    onConfirmDeleteAll,
    onConfirmExportRun,
    onConfirmExport,
    onConfirmImport
  }: Props = $props()
</script>

<Dialog
  open={dialogState.rename.open}
  onOpenChange={open => !open && onCloseRename()}
>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Rename Analysis</DialogTitle>
      <DialogDescription>Enter a new name for this analysis</DialogDescription>
    </DialogHeader>

    <div class="space-y-4 py-4">
      <div class="space-y-2">
        <Label for="new-name">Name</Label>
        <Input
          id="new-name"
          placeholder="My Analysis"
          value={dialogState.rename.newName}
          oninput={e => onRenameInput(e.currentTarget.value)}
          onkeydown={e => e.key === 'Enter' && onConfirmRename()}
        />
      </div>
    </div>

    <DialogFooter>
      <DialogClose>
        <Button variant="outline">Cancel</Button>
      </DialogClose>
      <Button onclick={onConfirmRename}>Save</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<Dialog
  open={dialogState.delete.open}
  onOpenChange={open => !open && onCloseDelete()}
>
  <DialogContent>
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2 text-red-600">
        <TriangleAlert class="h-5 w-5" />
        Delete Analysis
      </DialogTitle>
      <DialogDescription>
        Are you sure you want to delete "{dialogState.delete.item?.name ||
          'this analysis'}"? This action cannot be undone.
      </DialogDescription>
    </DialogHeader>

    <DialogFooter>
      <DialogClose>
        <Button variant="outline">Cancel</Button>
      </DialogClose>
      <Button variant="destructive" onclick={onConfirmDelete}>
        <Trash2 class="mr-2 h-4 w-4" />
        Delete
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<Dialog
  open={dialogState.deleteAll.open}
  onOpenChange={open => !open && onCloseDeleteAll()}
>
  <DialogContent>
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2 text-red-600">
        <TriangleAlert class="h-5 w-5" />
        Delete All History
      </DialogTitle>
      <DialogDescription>
        Are you sure you want to delete all {historyState.items.length} analyses?
        This action cannot be undone.
      </DialogDescription>
    </DialogHeader>

    <DialogFooter>
      <DialogClose>
        <Button variant="outline">Cancel</Button>
      </DialogClose>
      <Button variant="destructive" onclick={onConfirmDeleteAll}>
        <Trash2 class="mr-2 h-4 w-4" />
        Delete All
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<Dialog
  open={dialogState.exportRun.open}
  onOpenChange={open => !open && onCloseExportRun()}
>
  <DialogContent>
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <Download class="h-5 w-5" />
        Export Analysis Run
      </DialogTitle>
      <DialogDescription>
        Export "{dialogState.exportRun.item?.name || 'this analysis'}" and its
        page rows to a JSON file
      </DialogDescription>
    </DialogHeader>

    <div class="space-y-4 py-4">
      <div class="space-y-2">
        <Label for="run-export-path">File Path</Label>
        <Input
          id="run-export-path"
          placeholder="/home/user/pagelens-run-export.json"
          value={dialogState.exportRun.path}
          oninput={e => onExportRunInput(e.currentTarget.value)}
        />
        <p class="text-xs text-muted-foreground">
          Enter the full path where this run export file should be saved
        </p>
      </div>
    </div>

    <DialogFooter>
      <DialogClose>
        <Button variant="outline">Cancel</Button>
      </DialogClose>
      <Button onclick={onConfirmExportRun}>
        <Download class="mr-2 h-4 w-4" />
        Export Run
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<Dialog
  open={dialogState.export.open}
  onOpenChange={open => !open && onCloseExport()}
>
  <DialogContent>
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <Download class="h-5 w-5" />
        Export History
      </DialogTitle>
      <DialogDescription>Export all history to a JSON file</DialogDescription>
    </DialogHeader>

    <div class="space-y-4 py-4">
      <div class="space-y-2">
        <Label for="export-path">File Path</Label>
        <Input
          id="export-path"
          placeholder="/home/user/pagelens-export.json"
          value={dialogState.export.path}
          oninput={e => onExportInput(e.currentTarget.value)}
        />
        <p class="text-xs text-muted-foreground">
          Enter the full path where the export file should be saved
        </p>
      </div>
    </div>

    <DialogFooter>
      <DialogClose>
        <Button variant="outline">Cancel</Button>
      </DialogClose>
      <Button onclick={onConfirmExport}>
        <Download class="mr-2 h-4 w-4" />
        Export
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<Dialog
  open={dialogState.import.open}
  onOpenChange={open => !open && onCloseImport()}
>
  <DialogContent>
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <Upload class="h-5 w-5" />
        Import History
      </DialogTitle>
      <DialogDescription>Import analyses from a JSON file</DialogDescription>
    </DialogHeader>

    <div class="space-y-4 py-4">
      <div class="space-y-2">
        <Label for="import-path">File Path</Label>
        <Input
          id="import-path"
          placeholder="/home/user/pagelens-export.json"
          value={dialogState.import.path}
          oninput={e => onImportInput(e.currentTarget.value)}
        />
        <p class="text-xs text-muted-foreground">
          Enter the full path to the JSON file to import
        </p>
      </div>
    </div>

    <DialogFooter>
      <DialogClose>
        <Button variant="outline">Cancel</Button>
      </DialogClose>
      <Button onclick={onConfirmImport}>
        <Upload class="mr-2 h-4 w-4" />
        Import
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
