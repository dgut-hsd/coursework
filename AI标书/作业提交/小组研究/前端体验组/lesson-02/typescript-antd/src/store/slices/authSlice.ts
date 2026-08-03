import { createSlice, createAsyncThunk, type PayloadAction } from '@reduxjs/toolkit'

interface User {
  id: string
  name: string
  role: string
}

interface AuthState {
  user: User | null
  token: string | null
  isAuthenticated: boolean
  loading: boolean
  error: string | null
}

const initialState: AuthState = {
  user: null,
  token: null,
  isAuthenticated: false,
  loading: false,
  error: null,
}

// Mock login API
function mockLogin(username: string, password: string): Promise<{ user: User; token: string }> {
  return new Promise((resolve, reject) => {
    setTimeout(() => {
      if (username === 'admin' && password === '123456') {
        resolve({ user: { id: '1', name: '管理员', role: 'admin' }, token: 'mock-jwt-token-' + Date.now() })
      } else {
        reject(new Error('用户名或密码错误'))
      }
    }, 800)
  })
}

export const login = createAsyncThunk(
  'auth/login',
  async ({ username, password }: { username: string; password: string }, { rejectWithValue }) => {
    try {
      return await mockLogin(username, password)
    } catch (err) {
      return rejectWithValue((err as Error).message)
    }
  },
)

const authSlice = createSlice({
  name: 'auth',
  initialState,
  reducers: {
    logout: (state) => {
      state.user = null
      state.token = null
      state.isAuthenticated = false
    },
    setToken: (state, action: PayloadAction<string>) => {
      state.token = action.payload
    },
    clearError: (state) => {
      state.error = null
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(login.pending, (state) => {
        state.loading = true
        state.error = null
      })
      .addCase(login.fulfilled, (state, action) => {
        state.user = action.payload.user
        state.token = action.payload.token
        state.isAuthenticated = true
        state.loading = false
      })
      .addCase(login.rejected, (state, action) => {
        state.loading = false
        state.error = action.payload as string
      })
  },
})

export const { logout, setToken, clearError } = authSlice.actions
export default authSlice.reducer
