// Shared types between frontend and backend

export interface HealthResponse {
  status: string;
  version: string;
}

export interface ApiError {
  error: string;
  message: string;
}

// Add more shared types as needed
