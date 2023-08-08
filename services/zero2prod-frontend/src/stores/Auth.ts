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
  
      console.log('AuthStore::register')
      console.log(JSON.stringify(data, null, 2))

      let resp = null
      try {
        resp = await AuthService.register(data)
        // console.log(JSON.stringify(resp, null, 2))
      } catch (error) {
        // console.log('AuthStore::register error')
        // console.log(JSON.stringify(error, null, 2))

        throw new MyError('Failure, an unexpected error occured, please try again later.')
      }

      if ( resp?.data.status === 'fail' ) {
        // console.log('AuthStore::register error in resp')
        // console.log(JSON.stringify(resp?.data, null, 2))

        throw new MyError('Failure: ' + resp?.data.message)
      } else {
        // console.log('AuthStore::register storing access token')
        LocalStorage.setToken(resp.data?.token)
        this.isLoggedIn = true
      }
    },
    async authenticate() {
  
      console.log('AuthStore::authenticate')
      let resp = null
      try {
        resp = await AuthService.authenticate()
        console.log(JSON.stringify(resp, null, 2))
      } catch (error) {
        console.log('AuthStore::authenticate error')
        console.log(JSON.stringify(error, null, 2))
        this.isLoggedIn = false
        return
      }

      if ( resp?.data.status === 'fail' ) {
        console.log('AuthStore::authenticate error in resp')
        console.log(JSON.stringify(resp?.data, null, 2))
        this.isLoggedIn = false
      } else {
        this.isLoggedIn = true
      }
    }
  }
})
