<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useEmailStore } from '~/stores/emailStore'

const router = useRouter()
const route = useRoute()
const emailStore = useEmailStore()

const email = computed(() => {
  const id = route.params.id as string
  return emailStore.getEmailById(id)
})

onMounted(() => {
  if (email.value && !email.value.read) {
    emailStore.markAsRead(email.value.id)
  }
})

const formatDate = (dateString: string) => {
  const date = new Date(dateString)
  return date.toLocaleDateString('fr-FR', {
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

const formatFileSize = (bytes: number) => {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

const goBack = () => {
  router.back()
}

const reply = () => {
  if (!email.value) return
  router.push({
    path: '/compose',
    query: {
      reply: email.value.id,
      to: email.value.from.email,
      subject: `Re: ${email.value.subject}`
    }
  })
}

const replyAll = () => {
  if (!email.value) return
  const allRecipients = [
    email.value.from.email,
    ...email.value.to.map(a => a.email),
    ...(email.value.cc?.map(a => a.email) || [])
  ].filter((v, i, a) => a.indexOf(v) === i && v !== 'moi@example.com')

  router.push({
    path: '/compose',
    query: {
      reply: email.value.id,
      to: allRecipients.join(', '),
      subject: `Re: ${email.value.subject}`
    }
  })
}

const forward = () => {
  if (!email.value) return
  router.push({
    path: '/compose',
    query: {
      forward: email.value.id,
      subject: `Fwd: ${email.value.subject}`,
      body: `\n\n---------- Message transféré ----------\nDe: ${email.value.from.name} <${email.value.from.email}>\nDate: ${formatDate(email.value.date)}\nObjet: ${email.value.subject}\n\n${email.value.body}`
    }
  })
}

const deleteEmail = () => {
  if (!email.value) return
  emailStore.deleteEmail(email.value.id)
  router.back()
}

const toggleStar = () => {
  if (!email.value) return
  emailStore.toggleStar(email.value.id)
}
</script>

<template>
  <div v-if="email" class="flex h-full flex-col bg-gray-50">
    <div class="flex items-center justify-between border-b border-gray-200 bg-white px-4 py-3">
      <UButton variant="ghost" color="neutral" @click="goBack">
        <UIcon name="i-heroicons-arrow-left" />
      </UButton>
      <div class="flex gap-2">
        <UButton variant="ghost" color="neutral" @click="deleteEmail">
          <UIcon name="i-heroicons-trash" />
        </UButton>
        <UButton variant="ghost" color="neutral" @click="emailStore.markAsUnread(email.id)">
          <UIcon name="i-heroicons-envelope" />
        </UButton>
        <UButton variant="ghost" color="neutral" @click="toggleStar">
          <UIcon :name="email.starred ? 'i-heroicons-star-solid' : 'i-heroicons-star'" />
        </UButton>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto p-6">
      <UCard>
        <template #header>
          <div class="space-y-3">
            <h1 class="text-xl font-medium">{{ email.subject }}</h1>
            <div class="flex items-start justify-between gap-4">
              <div class="flex items-center gap-3">
                <UAvatar :text="(email.from.name || email.from.email).charAt(0).toUpperCase()" />
                <div>
                  <div class="text-sm font-medium">
                    {{ email.from.name || email.from.email }}
                    <span v-if="email.from.name" class="text-gray-500">&lt;{{ email.from.email }}&gt;</span>
                  </div>
                  <div class="text-xs text-gray-500">
                    à {{ email.to.map(a => a.name || a.email).join(', ') }}
                    <span v-if="email.cc && email.cc.length > 0">
                      , Cc: {{ email.cc.map(a => a.name || a.email).join(', ') }}
                    </span>
                  </div>
                </div>
              </div>
              <div class="text-xs text-gray-500">{{ formatDate(email.date) }}</div>
            </div>
          </div>
        </template>

        <div class="whitespace-pre-wrap text-sm text-gray-700">
          <div v-if="email.htmlBody" v-html="email.htmlBody"></div>
          <pre v-else class="whitespace-pre-wrap font-sans">{{ email.body }}</pre>
        </div>

        <div v-if="email.attachments && email.attachments.length > 0" class="mt-6">
          <div class="text-sm font-medium text-gray-500">Pièces jointes ({{ email.attachments.length }})</div>
          <div class="mt-3 grid grid-cols-1 gap-3 sm:grid-cols-2">
            <UCard v-for="attachment in email.attachments" :key="attachment.id" class="flex items-center gap-3">
              <div class="text-2xl">
                <UIcon name="i-heroicons-document" />
              </div>
              <div class="flex-1">
                <div class="text-sm font-medium">{{ attachment.filename }}</div>
                <div class="text-xs text-gray-500">{{ formatFileSize(attachment.size) }}</div>
              </div>
              <UButton variant="ghost" color="neutral">
                <UIcon name="i-heroicons-arrow-down-tray" />
              </UButton>
            </UCard>
          </div>
        </div>
      </UCard>
    </div>

    <div class="flex gap-2 border-t border-gray-200 bg-white px-4 py-3">
      <UButton variant="soft" color="primary" @click="reply">Répondre</UButton>
      <UButton variant="soft" color="primary" @click="replyAll">Répondre à tous</UButton>
      <UButton variant="soft" color="primary" @click="forward">Transférer</UButton>
    </div>
  </div>

  <div v-else class="flex h-full items-center justify-center bg-gray-50">
    <UCard class="text-center">
      <p class="text-gray-500">Email non trouvé</p>
      <UButton class="mt-3" color="primary" @click="goBack">Retour</UButton>
    </UCard>
  </div>
</template>
