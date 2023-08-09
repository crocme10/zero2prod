import { Get, Post } from '../utils/RestService'
const apiUrl = import.meta.env.VITE_API_URL

const AuthService = {
  register: (data: Map<string, any>) => Post(`${apiUrl}/register`, data),
  login: (data: Map<string, any>) => Post(`${apiUrl}/login`, data),
  logout: () => Get(`${apiUrl}/logout`),
  authenticate: () => Get(`${apiUrl}/authenticate`)
}

export default AuthService
