<script setup lang="ts">
import { computed, onMounted, watch } from 'vue'
import { useEmailStore } from '~/stores/emailStore'
import EmailList from '~/components/EmailList.vue'

const props = defineProps<{
  folder?: string
}>()

const route = useRoute()
const emailStore = useEmailStore()

const currentFolder = computed(() => {
  if (props.folder) return props.folder

  const path = route.path
  if (path === '/sent') return 'sent'
  if (path === '/drafts') return 'drafts'
  if (path === '/trash') return 'trash'
  return 'inbox'
})

const folderTitle = computed(() => {
  switch (currentFolder.value) {
    case 'sent': return 'Messages envoyés'
    case 'drafts': return 'Brouillons'
    case 'trash': return 'Corbeille'
    default: return 'Boîte de réception'
  }
})

const emails = computed(() => emailStore.emails.filter(e => e.folder === currentFolder.value))
const hasSelected = computed(() => emailStore.selectedEmails.length > 0)

watch(currentFolder, (newFolder) => {
  emailStore.setCurrentFolder(newFolder)
})

onMounted(() => {
  emailStore.setCurrentFolder(currentFolder.value)
})

const handleSelectAll = () => {
  if (hasSelected.value) {
    emailStore.deselectAll()
  } else {
    emailStore.selectAll()
  }
}

const handleDeleteSelected = () => {
  emailStore.deleteSelected()
}

const handleMarkAsRead = () => {
  emailStore.selectedEmails.forEach(e => emailStore.markAsRead(e.id))
  emailStore.deselectAll()
}

const handleMarkAsUnread = () => {
  emailStore.selectedEmails.forEach(e => emailStore.markAsUnread(e.id))
  emailStore.deselectAll()
}
</script>

<template>
  <div class="flex h-full flex-col bg-gray-50">
    <div class="flex items-center justify-between border-b border-gray-200 bg-white px-4 py-3">
      <div class="flex items-center gap-2">
        <UCheckbox :model-value="hasSelected" @update:modelValue="handleSelectAll" />
        <UButton variant="ghost" color="neutral" @click="emailStore.fetchEmails(true)">
          <UIcon name="i-heroicons-arrow-path" />
        </UButton>

        <template v-if="hasSelected">
          <UButton variant="ghost" color="neutral" @click="handleDeleteSelected">
            <UIcon name="i-heroicons-trash" />
          </UButton>
          <UButton variant="ghost" color="neutral" @click="handleMarkAsRead">
            <UIcon name="i-heroicons-envelope-open" />
          </UButton>
          <UButton variant="ghost" color="neutral" @click="handleMarkAsUnread">
            <UIcon name="i-heroicons-envelope" />
          </UButton>
        </template>
      </div>

      <div class="flex items-center gap-3">
        <h2 class="text-base font-medium">{{ folderTitle }}</h2>
        <UBadge color="neutral" variant="soft">{{ emails.length }} message(s)</UBadge>
      </div>

      <div class="flex items-center gap-2 text-sm text-gray-500">
        <span>1-{{ Math.min(50, emails.length) }} sur {{ emails.length }}</span>
        <UButton variant="ghost" color="neutral"><UIcon name="i-heroicons-chevron-left" /></UButton>
        <UButton variant="ghost" color="neutral"><UIcon name="i-heroicons-chevron-right" /></UButton>
      </div>
    </div>

    <div class="flex-1 overflow-hidden">
      <EmailList :emails="emails" />
    </div>
  </div>
</template>
