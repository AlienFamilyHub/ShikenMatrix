import { Route, Router } from "@solidjs/router";
import Admin from "./pages/Admin";
import Share from "./pages/Share";
import "./App.css";

export default function App() {
  return (
    <Router>
      <Route path="/" component={Share} />
      <Route path="/admin/*" component={Admin} />
    </Router>
  );
}
