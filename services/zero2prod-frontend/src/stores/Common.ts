import { defineStore } from 'pinia'

export const useCommonStore = defineStore('common', {
  state: () => ({}),
  actions: {
    showSuccessMess(message: String) {
      console.log(message)
    },
    showErrorMess(message: String) {
      console.log(message)
    }
  },
  getters: {}
})
