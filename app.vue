<template>
  <div class="flex h-screen w-screen bg-gray-50 text-gray-900">
    <Sidebar />
    <div class="flex min-w-0 flex-1 flex-col">
      <Header />
      <div class="flex-1 overflow-hidden">
        <NuxtPage />
      </div>
    </div>
  </div>
</template>
