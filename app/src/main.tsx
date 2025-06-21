import { Buffer } from "buffer";
window.Buffer = Buffer;

import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { WalletConnectionProvider } from './WalletConnectionProvider'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <WalletConnectionProvider>
      <App />
    </WalletConnectionProvider>
  </StrictMode>,
)
