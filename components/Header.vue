<script setup lang="ts">
import { ref } from 'vue'
import { useEmailStore } from '~/stores/emailStore'

const emailStore = useEmailStore()
const searchQuery = ref('')

const handleSearch = () => {
  emailStore.searchEmails(searchQuery.value)
}

const refreshEmails = async () => {
  await emailStore.fetchEmails(true)
}
</script>

<template>
  <header class="flex items-center justify-between gap-4 border-b border-gray-200 bg-white px-4 py-3">
    <div class="flex w-full max-w-xl items-center gap-2">
      <UInput
        v-model="searchQuery"
        placeholder="Rechercher dans les emails..."
        class="flex-1"
        @keydown.enter="handleSearch"
      >
        <template #leading>
          <UIcon name="i-heroicons-magnifying-glass" />
        </template>
      </UInput>
      <UButton variant="soft" color="primary" @click="handleSearch">
        Rechercher
      </UButton>
    </div>

    <div class="flex items-center gap-2">
      <UButton variant="ghost" color="neutral" @click="refreshEmails">
        <UIcon name="i-heroicons-arrow-path" />
      </UButton>
      <UButton variant="ghost" color="neutral">
        <UIcon name="i-heroicons-question-mark-circle" />
      </UButton>
      <UAvatar size="sm" icon="i-heroicons-user" />
    </div>
  </header>
</template>
