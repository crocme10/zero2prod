import axios from 'axios';
import LocalStorage from '../utils/LocalStorage'
// import { ResponseData } from '../types/Response'
// import { useCommonStore } from '../stores/Common'

const axiosInstance = axios.create({
  validateStatus: (status) => {
    return status >= 200 && status < 500
  }
})
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

// axiosInstance.interceptors.response.use(
//   response => {
//     console.log('AxiosInstance::response')
//     console.log(JSON.stringify(response, null, 2))
//     return response
//   },
//   error => {
//     console.log('AxiosInstance::error')
//     console.log(JSON.stringify(error, null, 2))
//     return Promise.reject(error)
//   }
// )
// 
