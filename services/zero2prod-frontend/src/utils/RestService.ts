import http from '../plugins/Axios'

export const Get = (apiUrl: string, query?: Map<string, any>) => {
  return http({
    method: 'GET',
    url: apiUrl,
    params: query ? Object.fromEntries(query) : null
  })
}

export const Post = (apiUrl: string, data: Map<string, any>, query?: Map<string, any>) => {
  return http({
    method: 'POST',
    url: apiUrl,
    data: Object.fromEntries(data),
    params: query ? Object.fromEntries(query) : null
  })
}

export const Put = (apiUrl: string, data: Map<string, any>) => {
  return http({
    method: 'PUT',
    url: apiUrl,
    data: Object.fromEntries(data)
  })
}

export const Delete = (apiUrl: string, query?: Map<string, any>, data?: Map<string, any>) => {
  return http({
    method: 'DELETE',
    url: apiUrl,
    params: query ? Object.fromEntries(query) : null,
    data: data ? Object.fromEntries(data) : null
  })
}

