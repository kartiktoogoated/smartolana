import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { WalletConnectionProvider } from "./WalletConnectionProvider";

const root = ReactDOM.createRoot(document.getElementById("root")!);
root.render(
  <React.StrictMode>
    <WalletConnectionProvider>
      <App />
    </WalletConnectionProvider>
  </React.StrictMode>
); 