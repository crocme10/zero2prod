import axiosInstance from '../plugins/Axios'

export const Get = (apiUrl: string, query?: Map<string, any>) => {
  return axiosInstance({
    method: 'GET',
    url: apiUrl,
    params: query ? Object.fromEntries(query) : null
  })
}

export const Post = (apiUrl: string, data: Map<string, any>, query?: Map<string, any>) => {
  return axiosInstance({
    method: 'POST',
    url: apiUrl,
    data: Object.fromEntries(data),
    params: query ? Object.fromEntries(query) : null
  })
}

export const Put = (apiUrl: string, data: Map<string, any>) => {
  return axiosInstance({
    method: 'PUT',
    url: apiUrl,
    data: Object.fromEntries(data)
  })
}

export const Delete = (apiUrl: string, query?: Map<string, any>, data?: Map<string, any>) => {
  return axiosInstance({
    method: 'DELETE',
    url: apiUrl,
    params: query ? Object.fromEntries(query) : null,
    data: data ? Object.fromEntries(data) : null
  })
}
