<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useEmailStore, type EmailAddress } from '~/stores/emailStore'

const router = useRouter()
const route = useRoute()
const emailStore = useEmailStore()

const to = ref('')
const cc = ref('')
const bcc = ref('')
const subject = ref('')
const body = ref('')
const showCc = ref(false)
const showBcc = ref(false)
const isSending = ref(false)
const draftId = ref<string | null>(null)
const autoSaveInterval = ref<number | null>(null)

const parseAddresses = (str: string): EmailAddress[] => {
  if (!str.trim()) return []

  return str.split(',').map(addr => {
    const trimmed = addr.trim()
    const match = trimmed.match(/^(.+?)\s*<(.+?)>$/)
    if (match) {
      return { name: match[1].trim(), email: match[2].trim() }
    }
    return { name: '', email: trimmed }
  }).filter(addr => addr.email)
}

onMounted(() => {
  const id = route.query.draft as string
  if (id) {
    const draft = emailStore.getEmailById(id)
    if (draft) {
      draftId.value = id
      to.value = draft.to.map(a => a.email).join(', ')
      cc.value = draft.cc?.map(a => a.email).join(', ') || ''
      bcc.value = draft.bcc?.map(a => a.email).join(', ') || ''
      subject.value = draft.subject
      body.value = draft.body

      if (cc.value) showCc.value = true
      if (bcc.value) showBcc.value = true
    }
  }

  autoSaveInterval.value = window.setInterval(saveDraft, 30000)
})

onUnmounted(() => {
  if (autoSaveInterval.value) {
    clearInterval(autoSaveInterval.value)
  }
})

const saveDraft = async () => {
  if (!to.value && !subject.value && !body.value) return

  await emailStore.saveDraft({
    id: draftId.value || undefined,
    from: { name: 'Moi', email: 'moi@example.com' },
    to: parseAddresses(to.value),
    cc: parseAddresses(cc.value),
    bcc: parseAddresses(bcc.value),
    subject: subject.value,
    body: body.value
  })

  if (!draftId.value) {
    draftId.value = Date.now().toString()
  }
}

const sendEmail = async () => {
  if (!to.value.trim()) {
    alert('Veuillez entrer au moins un destinataire')
    return
  }

  isSending.value = true

  try {
    await emailStore.sendEmail({
      from: { name: 'Moi', email: 'moi@example.com' },
      to: parseAddresses(to.value),
      cc: parseAddresses(cc.value),
      bcc: parseAddresses(bcc.value),
      subject: subject.value || '(Sans objet)',
      body: body.value
    })

    if (draftId.value) {
      emailStore.deleteEmail(draftId.value)
    }

    router.push('/sent')
  } catch (error) {
    console.error('Failed to send email:', error)
    alert('Erreur lors de l\'envoi du message')
  } finally {
    isSending.value = false
  }
}

const discardDraft = () => {
  if (draftId.value) {
    emailStore.deleteEmail(draftId.value)
  }
  router.back()
}

const close = () => {
  saveDraft()
  router.back()
}
</script>

<template>
  <div class="flex h-full flex-col bg-gray-50 dark:bg-gray-950">
    <div class="flex items-center justify-between border-b border-gray-200 bg-white px-4 py-3 dark:border-gray-800 dark:bg-gray-900">
      <h2 class="text-base font-medium">Nouveau message</h2>
      <UButton variant="ghost" color="neutral" @click="close">
        <UIcon name="i-heroicons-x-mark" />
        <span class="ml-1">Fermer</span>
      </UButton>
    </div>

    <div class="flex-1 overflow-y-auto p-6">
      <UCard class="space-y-4">
        <div class="flex items-center gap-3">
          <label class="w-14 text-sm text-gray-500">À</label>
          <UInput v-model="to" placeholder="destinataire@example.com" class="flex-1" />
          <div class="flex gap-2">
            <UButton v-if="!showCc" variant="ghost" color="neutral" @click="showCc = true">Cc</UButton>
            <UButton v-if="!showBcc" variant="ghost" color="neutral" @click="showBcc = true">Cci</UButton>
          </div>
        </div>

        <div v-if="showCc" class="flex items-center gap-3">
          <label class="w-14 text-sm text-gray-500">Cc</label>
          <UInput v-model="cc" placeholder="copie@example.com" class="flex-1" />
        </div>

        <div v-if="showBcc" class="flex items-center gap-3">
          <label class="w-14 text-sm text-gray-500">Cci</label>
          <UInput v-model="bcc" placeholder="copie-cachee@example.com" class="flex-1" />
        </div>

        <div class="flex items-center gap-3">
          <label class="w-14 text-sm text-gray-500">Objet</label>
          <UInput v-model="subject" placeholder="Objet du message" class="flex-1" />
        </div>

        <div>
          <UTextarea v-model="body" placeholder="Rédigez votre message..." :rows="12" />
        </div>
      </UCard>
    </div>

    <div class="flex items-center justify-between border-t border-gray-200 bg-white px-4 py-3 dark:border-gray-800 dark:bg-gray-900">
      <UButton color="primary" :loading="isSending" @click="sendEmail">
        <UIcon name="i-heroicons-paper-airplane" />
        <span class="ml-1">{{ isSending ? 'Envoi en cours...' : 'Envoyer' }}</span>
      </UButton>

      <div class="flex items-center gap-2">
        <UButton variant="ghost" color="neutral" @click="saveDraft">
          <UIcon name="i-heroicons-arrow-down-tray" />
          <span class="ml-1">Brouillon</span>
        </UButton>
        <UButton variant="ghost" color="neutral">
          <UIcon name="i-heroicons-paper-clip" />
          <span class="ml-1">Pièce jointe</span>
        </UButton>
        <UButton variant="ghost" color="neutral">
          <UIcon name="i-heroicons-link" />
          <span class="ml-1">Lien</span>
        </UButton>
        <UButton variant="ghost" color="neutral">
          <UIcon name="i-heroicons-photo" />
          <span class="ml-1">Image</span>
        </UButton>
        <UButton variant="ghost" color="neutral" @click="discardDraft">
          <UIcon name="i-heroicons-trash" />
          <span class="ml-1">Supprimer</span>
        </UButton>
      </div>
    </div>
  </div>
</template>
