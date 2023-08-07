import axios, { AxiosRequestConfig } from 'axios';
// import { ResponseData } from '../response-data/ResponseData';
import LocalStorage from '../utils/LocalStorage'
// import { useCommonStore } from '../store/common'

const http = axios.create({ })
export default http

http.interceptors.request.use(
  (config: AxiosRequestConfig) => {
    config.headers!['Accept'] = 'application/json'
    config.headers!['Content-Type'] = 'application/json'
    const token = LocalStorage.getToken()
    if (token) {
      config.headers!['Authorization'] = `Bearer ${token}`
    }
    if (config.url?.indexOf('login') === -1) {
      delete http.defaults.headers.common['Authorization']
    }
    return config
  },
  error => {
    Promise.resolve(false)
  }
)

// http.interceptors.response.use(
//   response => {
//     const commonStore = useCommonStore()
//     const _rs: ResponseData = new ResponseData(response.data)
//     
//     if (_rs.message) {
//       _rs.status ? commonStore.showSuccessMess(_rs.message) : commonStore.showErrorMess(_rs.message)
//     }
//     return _rs
//   },
//   error => {
//     return Promise.resolve(false)
//   }
// )

