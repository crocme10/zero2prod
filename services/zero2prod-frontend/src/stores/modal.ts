import { defineStore } from 'pinia'

export const useModalStore = defineStore('modal', {
  state: () => ({
    isOpen: false
  }),
  getters: {
    hiddenClass(state): string {
      return !state.isOpen ? "hidden": ""
    }
  },
  actions: {
    toggleHidden(): void {
      this.isOpen = !this.isOpen
    }
  }
})
