const LocalStorage = {
  getItem(name: string) {
    return localStorage.getItem(name)
  },
  setItem(name: string, value: string) {
    return localStorage.setItem(name, value)
  },
  setToken(value: string) {
    return localStorage.setItem('jwt', value)
  },
  getToken() {
    return localStorage.getItem('jwt')
  },
  removeItem(key: string) {
    return localStorage.removeItem(key)
  },
  removeToken() {
    return localStorage.removeItem('jwt')
  }
}

export default LocalStorage

