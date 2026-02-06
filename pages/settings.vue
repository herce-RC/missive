<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useEmailStore, type EmailAccount } from '~/stores/emailStore'

const router = useRouter()
const emailStore = useEmailStore()
const colorMode = useColorMode()

const activeTab = ref('accounts')
const showAddAccount = ref(false)
const editingAccountId = ref<string | null>(null)
const isEditing = computed(() => editingAccountId.value !== null)
const isTestingConnection = ref(false)
const testResult = ref<{ success: boolean; message: string } | null>(null)
const saveResult = ref<{ success: boolean; message: string } | null>(null)
const dbPath = ref<string | null>(null)

const newAccount = ref<Omit<EmailAccount, 'id'>>({
  email: '',
  name: '',
  imapServer: '',
  imapPort: 993,
  smtpServer: '',
  smtpPort: 587,
  username: '',
  password: '',
  useSsl: true,
  allowInvalidCerts: false,
  allowInvalidSmtpCerts: false
})

const resetForm = () => {
  newAccount.value = {
    email: '',
    name: '',
    imapServer: '',
    imapPort: 993,
    smtpServer: '',
    smtpPort: 587,
    username: '',
    password: '',
    useSsl: true,
    allowInvalidCerts: false,
    allowInvalidSmtpCerts: false
  }
  testResult.value = null
  saveResult.value = null
  editingAccountId.value = null
}


const openEditAccount = (account: EmailAccount) => {
  editingAccountId.value = account.id
  newAccount.value = {
    email: account.email,
    name: account.name,
    imapServer: account.imapServer,
    imapPort: account.imapPort,
    smtpServer: account.smtpServer,
    smtpPort: account.smtpPort,
    username: account.username,
    password: account.password,
    useSsl: account.useSsl,
    allowInvalidCerts: account.allowInvalidCerts,
    allowInvalidSmtpCerts: account.allowInvalidSmtpCerts
  }
  showAddAccount.value = true
}

const testConnection = async () => {
  isTestingConnection.value = true
  testResult.value = null

  try {
    const result = await invoke<{ success: boolean; message: string }>('test_connection', {
      account: {
        id: 'temp',
        email: newAccount.value.email,
        name: newAccount.value.name,
        imapServer: newAccount.value.imapServer,
        imapPort: newAccount.value.imapPort,
        smtpServer: newAccount.value.smtpServer,
        smtpPort: newAccount.value.smtpPort,
        username: newAccount.value.username || newAccount.value.email,
        password: newAccount.value.password,
        useSsl: newAccount.value.useSsl,
        allowInvalidCerts: newAccount.value.allowInvalidCerts,
        allowInvalidSmtpCerts: newAccount.value.allowInvalidSmtpCerts
      }
    })
    testResult.value = result
  } catch (error) {
    testResult.value = { success: false, message: 'Échec de la connexion. Vérifiez vos paramètres.' }
  } finally {
    isTestingConnection.value = false
  }
}

const saveAccount = async () => {
  if (!newAccount.value.email || !newAccount.value.imapServer || !newAccount.value.smtpServer) {
    alert('Veuillez remplir tous les champs obligatoires')
    return
  }

  try {
    if (isEditing.value && editingAccountId.value) {
      await emailStore.updateAccount({
        id: editingAccountId.value,
        ...newAccount.value
      })
      saveResult.value = { success: true, message: 'Compte mis à jour avec succès.' }
    } else {
      await emailStore.addAccount(newAccount.value)
      saveResult.value = { success: true, message: 'Compte enregistré avec succès.' }
    }
  } catch (error) {
    saveResult.value = { success: false, message: 'Échec de la sauvegarde du compte. Vérifiez vos paramètres et réessayez.' }
  }
}


const deleteAccount = async (id: string) => {
  if (confirm('Êtes-vous sûr de vouloir supprimer ce compte ?')) {
    await emailStore.removeAccount(id)
  }
}

const goBack = () => {
  router.back()
}

const autoFillProvider = (provider: string) => {
  switch (provider) {
    case 'gmail':
      newAccount.value.imapServer = 'imap.gmail.com'
      newAccount.value.imapPort = 993
      newAccount.value.smtpServer = 'smtp.gmail.com'
      newAccount.value.smtpPort = 587
      break
    case 'outlook':
      newAccount.value.imapServer = 'outlook.office365.com'
      newAccount.value.imapPort = 993
      newAccount.value.smtpServer = 'smtp.office365.com'
      newAccount.value.smtpPort = 587
      break
    case 'yahoo':
      newAccount.value.imapServer = 'imap.mail.yahoo.com'
      newAccount.value.imapPort = 993
      newAccount.value.smtpServer = 'smtp.mail.yahoo.com'
      newAccount.value.smtpPort = 587
      break
  }
}

const loadDbPath = async () => {
  try {
    dbPath.value = await invoke<string>('get_db_path')
  } catch (error) {
    dbPath.value = null
  }
}

const themeOptions = [
  { label: 'Clair', value: 'light' },
  { label: 'Sombre', value: 'dark' },
  { label: 'Système', value: 'system' }
]

const densityOptions = [
  { label: 'Confortable', value: 'comfortable' },
  { label: 'Compact', value: 'compact' }
]

const selectedTheme = ref(colorMode.preference || 'system')
const selectedDensity = ref('comfortable')


watch(showAddAccount, (open) => {
  if (!open) {
    resetForm()
  }
})

watch(selectedTheme, (value) => {
  colorMode.preference = value
})

loadDbPath()
</script>

<template>
  <div class="flex h-full flex-col bg-gray-50 dark:bg-gray-950">
    <div class="flex items-center gap-3 border-b border-gray-200 bg-white px-4 py-3 dark:border-gray-800 dark:bg-gray-900">
      <UButton variant="ghost" color="neutral" @click="goBack">
        <UIcon name="i-heroicons-arrow-left" />
      </UButton>
      <h1 class="text-lg font-medium">Paramètres</h1>
    </div>

    <div class="flex flex-1 overflow-hidden">
      <aside class="w-64 border-r border-gray-200 bg-white p-4 dark:border-gray-800 dark:bg-gray-900">
        <div class="space-y-2">
          <UButton :variant="activeTab === 'accounts' ? 'soft' : 'ghost'" color="primary" block @click="activeTab = 'accounts'">
            <template #leading><UIcon name="i-heroicons-envelope" /></template>
            Comptes email
          </UButton>
          <UButton :variant="activeTab === 'appearance' ? 'soft' : 'ghost'" color="primary" block @click="activeTab = 'appearance'">
            <template #leading><UIcon name="i-heroicons-paint-brush" /></template>
            Apparence
          </UButton>
          <UButton :variant="activeTab === 'notifications' ? 'soft' : 'ghost'" color="primary" block @click="activeTab = 'notifications'">
            <template #leading><UIcon name="i-heroicons-bell" /></template>
            Notifications
          </UButton>
          <UButton :variant="activeTab === 'about' ? 'soft' : 'ghost'" color="primary" block @click="activeTab = 'about'">
            <template #leading><UIcon name="i-heroicons-information-circle" /></template>
            À propos
          </UButton>
        </div>
      </aside>

      <div class="flex-1 overflow-y-auto p-6">
        <div v-if="activeTab === 'accounts'" class="space-y-6">
          <div class="flex items-center justify-between">
            <h2 class="text-lg font-medium">Comptes email</h2>
            <UButton color="primary" @click="showAddAccount = true">+ Ajouter un compte</UButton>
          </div>

          <div class="space-y-3">
            <UCard v-for="account in emailStore.accounts" :key="account.id">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <UAvatar :text="account.email.charAt(0).toUpperCase()" />
                  <div>
                    <div class="text-sm font-medium">{{ account.name || account.email }}</div>
                    <div class="text-xs text-gray-500">{{ account.email }}</div>
                  </div>
                </div>
                <div class="flex flex-wrap items-center gap-2">
                  <UButton variant="soft" color="primary" size="sm" @click="openEditAccount(account)">
                    <UIcon name="i-heroicons-pencil" />
                    <span class="ml-1">Modifier</span>
                  </UButton>
                  <UButton variant="soft" color="red" size="sm" @click="deleteAccount(account.id)">
                    <UIcon name="i-heroicons-trash" />
                    <span class="ml-1">Supprimer</span>
                  </UButton>
                </div>
              </div>
            </UCard>

            <UCard v-if="emailStore.accounts.length === 0" class="text-center">
              <p class="text-gray-500">Aucun compte configuré</p>
              <UButton class="mt-3" color="primary" @click="showAddAccount = true">
                Ajouter votre premier compte
              </UButton>
            </UCard>
          </div>
        </div>

        <div v-if="activeTab === 'appearance'" class="space-y-4">
          <h2 class="text-lg font-medium">Apparence</h2>
          <UCard class="space-y-4">
            <div class="space-y-2">
              <label class="text-sm text-gray-600">Thème</label>
              <USelect v-model="selectedTheme" :items="themeOptions" />
            </div>
            <div class="space-y-2">
              <label class="text-sm text-gray-600">Densité d'affichage</label>
              <USelect v-model="selectedDensity" :items="densityOptions" />
            </div>
          </UCard>
        </div>

        <div v-if="activeTab === 'notifications'" class="space-y-4">
          <h2 class="text-lg font-medium">Notifications</h2>
          <UCard class="space-y-3">
            <UCheckbox label="Notifications de nouveaux messages" :model-value="true" />
            <UCheckbox label="Son de notification" :model-value="true" />
            <UCheckbox label="Notifications sur le bureau" />
          </UCard>
        </div>

        <div v-if="activeTab === 'about'" class="space-y-4">
          <h2 class="text-lg font-medium">À propos</h2>
          <UCard>
            <div class="space-y-1 text-sm">
              <p><strong>Email Client</strong></p>
              <p>Version 0.1.0</p>
              <p>Développé avec Tauri, Nuxt et Rust</p>
              <p>Base de données: SurrealDB</p>
              <p v-if="dbPath"><strong>Chemin DB:</strong> {{ dbPath }}</p>
            </div>
          </UCard>
        </div>
      </div>
    </div>

    <UModal v-model="showAddAccount">
      <UCard class="w-full max-w-2xl">
        <template #header>
          <div class="flex items-center justify-between">
            <h2 class="text-lg font-medium">{{ isEditing ? 'Modifier un compte email' : 'Ajouter un compte email' }}</h2>
            <UButton variant="ghost" color="neutral" @click="showAddAccount = false">
              <UIcon name="i-heroicons-x-mark" />
            </UButton>
          </div>
        </template>

        <div class="space-y-4">
          <div class="space-y-2">
            <p class="text-sm text-gray-500">Configuration rapide :</p>
            <div class="flex gap-2">
              <UButton variant="soft" color="primary" @click="autoFillProvider('gmail')">Gmail</UButton>
              <UButton variant="soft" color="primary" @click="autoFillProvider('outlook')">Outlook</UButton>
              <UButton variant="soft" color="primary" @click="autoFillProvider('yahoo')">Yahoo</UButton>
            </div>
          </div>

          <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <div class="space-y-2">
              <label class="text-sm text-gray-600">Nom affiché</label>
              <UInput v-model="newAccount.name" placeholder="Mon nom" />
            </div>
            <div class="space-y-2">
              <label class="text-sm text-gray-600">Adresse email *</label>
              <UInput v-model="newAccount.email" type="email" placeholder="email@example.com" />
            </div>
            <div class="space-y-2">
              <label class="text-sm text-gray-600">Nom d'utilisateur</label>
              <UInput v-model="newAccount.username" placeholder="Nom d'utilisateur" />
            </div>
            <div class="space-y-2">
              <label class="text-sm text-gray-600">Mot de passe *</label>
              <UInput v-model="newAccount.password" type="password" placeholder="Mot de passe" />
            </div>
            <div class="space-y-2">
              <label class="text-sm text-gray-600">Serveur IMAP *</label>
              <UInput v-model="newAccount.imapServer" placeholder="imap.example.com" />
            </div>
            <div class="space-y-2">
              <label class="text-sm text-gray-600">Port IMAP</label>
              <UInput v-model.number="newAccount.imapPort" type="number" />
            </div>
            <div class="space-y-2">
              <label class="text-sm text-gray-600">Serveur SMTP *</label>
              <UInput v-model="newAccount.smtpServer" placeholder="smtp.example.com" />
            </div>
            <div class="space-y-2">
              <label class="text-sm text-gray-600">Port SMTP</label>
              <UInput v-model.number="newAccount.smtpPort" type="number" />
            </div>
          </div>

          <div class="space-y-2">
            <UCheckbox v-model="newAccount.useSsl" label="Utiliser SSL/TLS" />
            <UCheckbox v-model="newAccount.allowInvalidCerts" label="Ignorer la vérification SSL (IMAP)" />
            <UCheckbox v-model="newAccount.allowInvalidSmtpCerts" label="Ignorer la vérification SSL (SMTP)" />
          </div>

        </div>

        <template #footer>
          <div class="flex flex-col gap-3">
            <UAlert v-if="testResult" :color="testResult.success ? 'success' : 'error'" variant="solid">
              <div class="text-sm font-medium">{{ testResult.success ? 'Connexion réussie' : 'Connexion échouée' }}</div>
              <div class="text-xs whitespace-pre-wrap opacity-90">{{ testResult.message }}</div>
            </UAlert>
            <UAlert v-if="saveResult" :color="saveResult.success ? 'success' : 'error'" variant="soft">
              <div class="text-sm font-medium">{{ saveResult.success ? 'Enregistrement réussi' : 'Enregistrement échoué' }}</div>
              <div class="text-xs whitespace-pre-wrap">{{ saveResult.message }}</div>
            </UAlert>

            <div class="flex items-center justify-end gap-2">
              <UButton variant="soft" color="primary" :loading="isTestingConnection" @click="testConnection">
                {{ isTestingConnection ? 'Test en cours...' : 'Tester la connexion' }}
              </UButton>
              <UButton color="primary" @click="saveAccount">Enregistrer</UButton>
            </div>
          </div>
        </template>
      </UCard>
    </UModal>
  </div>
</template>
