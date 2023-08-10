import { defineStore } from 'pinia'
import LocalStorage from '../utils/LocalStorage'
import AuthService from '../api/AuthService'
import { MyError } from '../types/Error'

export const useAuthStore = defineStore('auth', {
  state: () => ({
    isLoggedIn: false
  }),
  actions: {
    async register(data: Map<string, any>) {
      // console.log('AuthStore::register -> start')
      // console.log(JSON.stringify(data, null, 2))

      let resp = null
      try {
        resp = await AuthService.register(data)
      } catch (error) {
        throw new MyError('Failure, an unexpected error occured, please try again later.')
      }

      if (resp.status != 200) {
        this.isLoggedIn = false
        if (resp?.data.status === 'fail') {
          throw new MyError('Failure: ' + resp?.data.message)
        } else {
          throw new MyError('Failure, an unexpected error occured, please try again later.')
        }
      }
      if (resp?.data.status !== 'success') {
        throw new MyError('Failure, an unexpected error occured, please try again later.')
      } else {
        LocalStorage.setToken(resp.data?.token)
        this.isLoggedIn = true
      }
    },
    async login(data: Map<string, any>) {
      let resp = null
      try {
        resp = await AuthService.login(data)
      } catch (error) {
        throw new MyError('Failure, an unexpected error occured, please try again later.')
      }
      if (resp.status != 200) {
        this.isLoggedIn = false
        if (resp?.data.status === 'fail') {
          throw new MyError('Failure: ' + resp?.data.message)
        } else {
          throw new MyError('Failure, an unexpected error occured, please try again later.')
        }
      }
      if (resp?.data.status !== 'success') {
        // That should not happen: status code is 200, but status message is not 'success'
        throw new MyError('Failure, an unexpected error occured, please try again later.')
      } else {
        LocalStorage.setToken(resp.data?.token)
        this.isLoggedIn = true
      }
    },
    async authenticate() {
      let resp = null
      try {
        resp = await AuthService.authenticate()
      } catch (error) {
        this.isLoggedIn = false
        return
      }

      if (resp.status != 200) {
        this.isLoggedIn = false
        return
      }
      if (resp?.data.status === 'success') {
        // console.log('AuthStore::authenticate error in resp')
        // console.log(JSON.stringify(resp?.data, null, 2))
        this.isLoggedIn = true
      } else {
        this.isLoggedIn = false
      }
    },
    async logout() {
      try {
        await AuthService.logout()
      } catch (error) {
        return
      }

      LocalStorage.removeToken()
      this.isLoggedIn = false
    }
  }
})
