import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface EmailAddress {
  name: string
  email: string
}

export interface Attachment {
  id: string
  filename: string
  size: number
  mimeType: string
}

export interface Email {
  id: string
  from: EmailAddress
  to: EmailAddress[]
  cc?: EmailAddress[]
  bcc?: EmailAddress[]
  subject: string
  body: string
  htmlBody?: string
  date: string
  read: boolean
  starred: boolean
  selected: boolean
  folder: string
  attachments?: Attachment[]
}

export interface EmailAccount {
  id: string
  email: string
  name: string
  imapServer: string
  imapPort: number
  smtpServer: string
  smtpPort: number
  username: string
  password: string
  useSsl: boolean
  allowInvalidCerts: boolean
  allowInvalidSmtpCerts: boolean
  userId?: string | null
}

export const useEmailStore = defineStore('email', () => {
  const emails = ref<Email[]>([])
  const currentEmail = ref<Email | null>(null)
  const currentFolder = ref('inbox')
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const isSyncing = ref(false)
  const syncStatus = ref<{ type: 'success' | 'error'; message: string } | null>(null)
  const searchQuery = ref('')
  const accounts = ref<EmailAccount[]>([])
  const currentAccount = ref<EmailAccount | null>(null)

  let hasSeenAnyEmail = false

  const playNotificationSound = () => {
    if (typeof window === 'undefined') return
    try {
      const AudioContextClass = window.AudioContext || (window as any).webkitAudioContext
      if (!AudioContextClass) return
      const ctx = new AudioContextClass()
      const osc = ctx.createOscillator()
      const gain = ctx.createGain()
      osc.type = 'sine'
      osc.frequency.value = 880
      gain.gain.value = 0.001
      osc.connect(gain)
      gain.connect(ctx.destination)
      const now = ctx.currentTime
      gain.gain.exponentialRampToValueAtTime(0.06, now + 0.02)
      gain.gain.exponentialRampToValueAtTime(0.001, now + 0.16)
      osc.start(now)
      osc.stop(now + 0.18)
    } catch {
      // ignore audio errors
    }
  }

  const markSeenIfAny = (list: Email[]) => {
    if (list.length > 0) hasSeenAnyEmail = true
  }

  // Computed properties
  const filteredEmails = computed(() => {
    let result = emails.value.filter(e => e.folder === currentFolder.value)

    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      result = result.filter(e =>
        e.subject.toLowerCase().includes(query) ||
        e.body.toLowerCase().includes(query) ||
        e.from.email.toLowerCase().includes(query) ||
        e.from.name.toLowerCase().includes(query)
      )
    }

    return result.sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime())
  })

  const unreadCount = computed(() =>
    emails.value.filter(e => !e.read && e.folder === 'inbox').length
  )

  const draftsCount = computed(() =>
    emails.value.filter(e => e.folder === 'drafts').length
  )

  const selectedEmails = computed(() =>
    emails.value.filter(e => e.selected)
  )

  // Actions
  async function fetchEmails(forceSync = false, skipAutoSync = false) {
    isLoading.value = true
    error.value = null

    try {
      const result = await invoke<Email[]>('fetch_emails', {
        folder: currentFolder.value
      })
      emails.value = result.map(e => ({ ...e, selected: false }))
      markSeenIfAny(emails.value)

      if (!skipAutoSync && (forceSync || result.length === 0)) {
        await syncEmails()
      }
    } catch (e) {
      error.value = String(e)
      if (accounts.value.length === 0) {
        loadMockData()
      }
    } finally {
      isLoading.value = false
    }
  }

  function loadMockData() {
    emails.value = [
      {
        id: '1',
        from: { name: 'Jean Dupont', email: 'jean.dupont@example.com' },
        to: [{ name: 'Moi', email: 'moi@example.com' }],
        subject: 'Réunion de projet demain',
        body: 'Bonjour,\n\nJe vous rappelle que nous avons une réunion de projet demain à 10h.\n\nCordialement,\nJean',
        date: new Date().toISOString(),
        read: false,
        starred: true,
        selected: false,
        folder: 'inbox'
      },
      {
        id: '2',
        from: { name: 'Marie Martin', email: 'marie.martin@example.com' },
        to: [{ name: 'Moi', email: 'moi@example.com' }],
        subject: 'Documents à signer',
        body: 'Bonjour,\n\nVeuillez trouver ci-joint les documents à signer.\n\nMerci,\nMarie',
        date: new Date(Date.now() - 3600000).toISOString(),
        read: false,
        starred: false,
        selected: false,
        folder: 'inbox',
        attachments: [{ id: '1', filename: 'document.pdf', size: 1024000, mimeType: 'application/pdf' }]
      },
      {
        id: '3',
        from: { name: 'Support Technique', email: 'support@example.com' },
        to: [{ name: 'Moi', email: 'moi@example.com' }],
        subject: 'Votre ticket #12345 a été résolu',
        body: 'Bonjour,\n\nVotre ticket de support a été résolu. N\'hésitez pas à nous contacter si vous avez d\'autres questions.\n\nL\'équipe Support',
        date: new Date(Date.now() - 86400000).toISOString(),
        read: true,
        starred: false,
        selected: false,
        folder: 'inbox'
      },
      {
        id: '4',
        from: { name: 'Newsletter', email: 'newsletter@example.com' },
        to: [{ name: 'Moi', email: 'moi@example.com' }],
        subject: 'Les actualités de la semaine',
        body: 'Découvrez les dernières actualités et tendances de la semaine dans notre newsletter hebdomadaire.',
        date: new Date(Date.now() - 172800000).toISOString(),
        read: true,
        starred: false,
        selected: false,
        folder: 'inbox'
      },
      {
        id: '5',
        from: { name: 'Moi', email: 'moi@example.com' },
        to: [{ name: 'Client', email: 'client@example.com' }],
        subject: 'Proposition commerciale',
        body: 'Bonjour,\n\nVeuillez trouver ci-joint notre proposition commerciale.\n\nCordialement',
        date: new Date(Date.now() - 259200000).toISOString(),
        read: true,
        starred: false,
        selected: false,
        folder: 'sent'
      }
    ]
  }

  async function syncEmails(): Promise<{ total: number; newCount: number }> {
    const account = currentAccount.value || accounts.value[0]
    if (!account) return { total: 0, newCount: 0 }

    try {
      const existingIds = new Set(emails.value.map(e => e.id))
      const synced = await invoke<Email[]>('sync_emails', {
        accountId: account.id,
        folder: currentFolder.value
      })
      if (synced && synced.length > 0) {
        const mapped = synced.map(e => ({ ...e, selected: false }))
        const newItems = mapped.filter(e => !existingIds.has(e.id))
        if (newItems.length > 0 && existingIds.size > 0) {
          playNotificationSound()
        }
        emails.value = mapped
        markSeenIfAny(emails.value)
        return { total: mapped.length, newCount: newItems.length }
      }
      return { total: 0, newCount: 0 }
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function refreshEmails() {
    syncStatus.value = null
    isSyncing.value = true

    if (accounts.value.length === 0) {
      syncStatus.value = { type: 'error', message: 'Aucun compte configuré.' }
      isSyncing.value = false
      return
    }

    try {
      const existingIds = new Set(emails.value.map(e => e.id))
      let totalSynced = 0
      let totalNew = 0

      for (const account of accounts.value) {
        const synced = await invoke<Email[]>('sync_emails', {
          accountId: account.id,
          folder: currentFolder.value
        })
        totalSynced += synced.length
        const newItems = synced.filter(e => !existingIds.has(e.id))
        if (newItems.length > 0 && existingIds.size > 0) {
          playNotificationSound()
        }
        for (const item of newItems) {
          existingIds.add(item.id)
        }
        totalNew += newItems.length
      }

      const result = await invoke<Email[]>('fetch_emails', { folder: currentFolder.value })
      emails.value = result.map(e => ({ ...e, selected: false }))
      markSeenIfAny(emails.value)

      syncStatus.value = {
        type: 'success',
        message: `Synchronisation OK (${accounts.value.length} compte(s)) : ${totalNew} nouveau(x), ${totalSynced} récupéré(s).`
      }
    } catch (e) {
      error.value = String(e)
      syncStatus.value = {
        type: 'error',
        message: `Échec de la synchronisation : ${String(e)}`
      }
    } finally {
      isSyncing.value = false
    }
  }

  async function sendEmail(email: Omit<Email, 'id' | 'date' | 'read' | 'starred' | 'selected' | 'folder'>) {
    isLoading.value = true
    error.value = null

    try {
      await invoke('send_email', { email })
      const sentEmail: Email = {
        ...email,
        id: Date.now().toString(),
        date: new Date().toISOString(),
        read: true,
        starred: false,
        selected: false,
        folder: 'sent'
      }
      emails.value.push(sentEmail)
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function saveDraft(email: Partial<Email>) {
    const draft: Email = {
      id: email.id || Date.now().toString(),
      from: email.from || { name: '', email: '' },
      to: email.to || [],
      subject: email.subject || '',
      body: email.body || '',
      date: new Date().toISOString(),
      read: true,
      starred: false,
      selected: false,
      folder: 'drafts'
    }

    const existingIndex = emails.value.findIndex(e => e.id === draft.id)
    if (existingIndex >= 0) {
      emails.value[existingIndex] = draft
    } else {
      emails.value.push(draft)
    }
  }

  function markAsRead(id: string) {
    const email = emails.value.find(e => e.id === id)
    if (email) {
      email.read = true
      invoke('mark_as_read', { id }).catch(console.error)
    }
  }

  function markAsUnread(id: string) {
    const email = emails.value.find(e => e.id === id)
    if (email) {
      email.read = false
      invoke('mark_as_unread', { id }).catch(console.error)
    }
  }

  function toggleStar(id: string) {
    const email = emails.value.find(e => e.id === id)
    if (email) {
      email.starred = !email.starred
      invoke('toggle_star', { id, starred: email.starred }).catch(console.error)
    }
  }

  function toggleSelect(id: string) {
    const email = emails.value.find(e => e.id === id)
    if (email) {
      email.selected = !email.selected
    }
  }

  function selectAll() {
    filteredEmails.value.forEach(e => e.selected = true)
  }

  function deselectAll() {
    emails.value.forEach(e => e.selected = false)
  }

  async function deleteEmail(id: string) {
    const email = emails.value.find(e => e.id === id)
    if (email) {
      if (email.folder === 'trash') {
        emails.value = emails.value.filter(e => e.id !== id)
        invoke('delete_email', { id }).catch(console.error)
      } else {
        email.folder = 'trash'
        invoke('move_to_trash', { id }).catch(console.error)
      }
    }
  }

  async function deleteSelected() {
    const selected = selectedEmails.value
    for (const email of selected) {
      await deleteEmail(email.id)
    }
    deselectAll()
  }

  function moveToFolder(id: string, folder: string) {
    const email = emails.value.find(e => e.id === id)
    if (email) {
      email.folder = folder
      invoke('move_to_folder', { id, folder }).catch(console.error)
    }
  }

  function setCurrentFolder(folder: string) {
    currentFolder.value = folder
    deselectAll()
  }

  function setCurrentEmail(email: Email | null) {
    currentEmail.value = email
  }

  function searchEmails(query: string) {
    searchQuery.value = query
  }

  function getEmailById(id: string): Email | undefined {
    return emails.value.find(e => e.id === id)
  }

  // Account management
  async function addAccount(account: Omit<EmailAccount, 'id'>) {
    const newAccount: EmailAccount = {
      ...account,
      id: Date.now().toString()
    }

    try {
      const saved = await invoke<EmailAccount>('save_account', { account: newAccount })
      await loadAccounts()
      const matched = accounts.value.find(a => a.id === saved.id)
      if (matched) {
        currentAccount.value = matched
      }
      await refreshEmails()
    } catch (e) {
      console.error('Failed to save account:', e)
      throw e
    }
  }


  async function updateAccount(account: EmailAccount) {
    try {
      const saved = await invoke<EmailAccount>('save_account', { account })
      await loadAccounts()
      const matched = accounts.value.find(a => a.id === saved.id)
      if (matched) {
        currentAccount.value = matched
      }
    } catch (e) {
      console.error('Failed to update account:', e)
      throw e
    }
  }

  async function removeAccount(id: string) {
    accounts.value = accounts.value.filter(a => a.id !== id)

    try {
      await invoke('remove_account', { id })
      await loadAccounts()
      if (currentAccount.value?.id === id) {
        currentAccount.value = accounts.value[0] || null
      }
    } catch (e) {
      console.error('Failed to remove account:', e)
      throw e
    }
  }

  function setCurrentAccount(account: EmailAccount | null) {
    currentAccount.value = account
  }

  async function loadAccounts() {
    try {
      const result = await invoke<EmailAccount[]>('get_accounts')
      accounts.value = result
      if (!currentAccount.value && accounts.value.length > 0) {
        currentAccount.value = accounts.value[0]
      }
      if (accounts.value.length > 0) {
        await refreshEmails()
      }
    } catch (e) {
      console.error('Failed to load accounts:', e)
    }
  }

  // Initialize
  loadAccounts()

  return {
    // State
    emails,
    currentEmail,
    currentFolder,
    isLoading,
    error,
    isSyncing,
    syncStatus,
    searchQuery,
    accounts,
    currentAccount,

    // Computed
    filteredEmails,
    unreadCount,
    draftsCount,
    selectedEmails,

    // Actions
    fetchEmails,
    syncEmails,
    refreshEmails,
    sendEmail,
    saveDraft,
    markAsRead,
    markAsUnread,
    toggleStar,
    toggleSelect,
    selectAll,
    deselectAll,
    deleteEmail,
    deleteSelected,
    moveToFolder,
    setCurrentFolder,
    setCurrentEmail,
    searchEmails,
    getEmailById,
    addAccount,
    removeAccount,
    setCurrentAccount,
    loadAccounts
  }
})
