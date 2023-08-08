import axios from 'axios';
import LocalStorage from '../utils/LocalStorage'
import { ResponseData } from '../types/Response'
import { useCommonStore } from '../stores/Common'

const axiosInstance = axios.create({ })
export default axiosInstance

axiosInstance.interceptors.request.use(
  config => {
    config.headers!['Accept'] = 'application/json'
    config.headers!['Content-Type'] = 'application/json'
    const token = LocalStorage.getToken()
    if (token) {
      config.headers!['Authorization'] = `Bearer ${token}`
    }
    if (config.url?.indexOf('login') === -1) {
      delete axiosInstance.defaults.headers.common['Authorization']
    }
    return config
  },
  error => {
    return Promise.reject(error)
  }
)

axiosInstance.interceptors.response.use(
  response => {
    const commonStore = useCommonStore()
    const _rs: ResponseData = new ResponseData(response.data)
    
    if (_rs.message) {
      _rs.status ? commonStore.showSuccessMess(_rs.message) : commonStore.showErrorMess(_rs.message)
    }
    return response
  },
  error => {
    return Promise.reject(error)
  }
)

