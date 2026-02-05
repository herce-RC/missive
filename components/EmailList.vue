<script setup lang="ts">
import { useEmailStore, type Email } from '~/stores/emailStore'

const props = defineProps<{
  emails: Email[]
}>()

const router = useRouter()
const emailStore = useEmailStore()

const formatDate = (dateString: string) => {
  const date = new Date(dateString)
  const now = new Date()
  const isToday = date.toDateString() === now.toDateString()

  if (isToday) {
    return date.toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' })
  }

  const isThisYear = date.getFullYear() === now.getFullYear()
  if (isThisYear) {
    return date.toLocaleDateString('fr-FR', { day: 'numeric', month: 'short' })
  }

  return date.toLocaleDateString('fr-FR', { day: 'numeric', month: 'short', year: 'numeric' })
}

const truncateText = (text: string, maxLength: number) => {
  if (text.length <= maxLength) return text
  return text.substring(0, maxLength) + '...'
}

const openEmail = (email: Email) => {
  if (!email.read) {
    emailStore.markAsRead(email.id)
  }
  router.push(`/email/${email.id}`)
}

const toggleStar = (email: Email, event: Event) => {
  event.stopPropagation()
  emailStore.toggleStar(email.id)
}

const toggleSelect = (email: Email, value: boolean | 'indeterminate') => {
  if (value === 'indeterminate') return
  emailStore.toggleSelect(email.id)
}
</script>

<template>
  <div class="h-full overflow-y-auto">
    <div
      v-for="email in emails"
      :key="email.id"
      class="flex cursor-pointer items-center gap-3 border-b border-gray-200 px-4 py-3 transition hover:bg-gray-50"
      :class="{ 'bg-primary-50 font-semibold': !email.read }"
      @click="openEmail(email)"
    >
      <UCheckbox
        :model-value="email.selected"
        @update:modelValue="toggleSelect(email, $event)"
        @click.stop
      />

      <UButton variant="ghost" color="neutral" square @click="toggleStar(email, $event)">
        <UIcon :name="email.starred ? 'i-heroicons-star-solid' : 'i-heroicons-star'" />
      </UButton>

      <div class="w-48 truncate text-sm">
        {{ email.from.name || email.from.email }}
      </div>

      <div class="flex min-w-0 flex-1 items-center gap-2 text-sm">
        <span class="truncate">{{ email.subject }}</span>
        <span class="text-gray-400">-</span>
        <span class="truncate text-gray-500">{{ truncateText(email.body, 80) }}</span>
      </div>

      <UBadge v-if="email.attachments && email.attachments.length > 0" color="neutral" variant="soft">
        <UIcon name="i-heroicons-paper-clip" />
      </UBadge>

      <div class="ml-2 w-20 text-right text-xs text-gray-500">
        {{ formatDate(email.date) }}
      </div>
    </div>

    <div v-if="emails.length === 0" class="flex h-full flex-col items-center justify-center p-10 text-gray-500">
      <div class="text-5xl">
        <UIcon name="i-heroicons-inbox" />
      </div>
      <p class="mt-3">Aucun email dans ce dossier</p>
    </div>
  </div>
</template>

<style scoped>
</style>
