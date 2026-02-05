<script setup lang="ts">
import { computed } from 'vue'
import { useEmailStore } from '~/stores/emailStore'

const router = useRouter()
const route = useRoute()
const emailStore = useEmailStore()

const folders = [
  { name: 'Boîte de réception', icon: 'i-heroicons-inbox', route: '/', badge: computed(() => emailStore.unreadCount) },
  { name: 'Envoyés', icon: 'i-heroicons-paper-airplane', route: '/sent', badge: null },
  { name: 'Brouillons', icon: 'i-heroicons-pencil-square', route: '/drafts', badge: computed(() => emailStore.draftsCount) },
  { name: 'Corbeille', icon: 'i-heroicons-trash', route: '/trash', badge: null }
]

const isActive = (folderRoute: string) => route.path === folderRoute

const navigateTo = (folderRoute: string) => {
  router.push(folderRoute)
}

const openCompose = () => {
  router.push('/compose')
}
</script>

<template>
  <aside class="flex h-full w-64 flex-col border-r border-gray-200 bg-white p-4">
    <div class="mb-6">
      <h1 class="text-xl font-semibold">Email</h1>
    </div>

    <UButton class="mb-4" color="primary" variant="solid" block @click="openCompose">
      <template #leading>
        <UIcon name="i-heroicons-pencil-square" />
      </template>
      Nouveau message
    </UButton>

    <div class="flex-1 space-y-1">
      <UButton
        v-for="folder in folders"
        :key="folder.route"
        :variant="isActive(folder.route) ? 'soft' : 'ghost'"
        :color="isActive(folder.route) ? 'primary' : 'neutral'"
        block
        class="justify-between"
        @click="navigateTo(folder.route)"
      >
        <span class="flex items-center gap-2">
          <UIcon :name="folder.icon" />
          <span>{{ folder.name }}</span>
        </span>
        <UBadge v-if="folder.badge && folder.badge.value > 0" color="primary" variant="solid">
          {{ folder.badge.value }}
        </UBadge>
      </UButton>
    </div>

    <div class="mt-4 border-t border-gray-200 pt-4">
      <UButton variant="ghost" color="neutral" block @click="router.push('/settings')">
        <template #leading>
          <UIcon name="i-heroicons-cog-6-tooth" />
        </template>
        Paramètres
      </UButton>
    </div>
  </aside>
</template>
