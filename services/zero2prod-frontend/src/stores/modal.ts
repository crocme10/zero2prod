import { defineStore } from 'pinia'

export const useModalStore = defineStore({
  id: 'store',
  state: () => ({
    isOpen: false
  })
})
