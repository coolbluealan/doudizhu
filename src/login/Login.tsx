import "./login.css";

import { useEffect } from "react";
import { Form, useSearchParams } from "react-router";

import FormError from "@/FormError";

export default function Login() {
  const [searchParams] = useSearchParams();
  const to = searchParams.get("next") || "/";

  useEffect(() => {
    document.title = "Login";
  }, []);

  return (
    <div className="center-container">
      <h1>Enter a username</h1>
      <Form method="POST">
        <div className="form-row">
          <input
            type="text"
            name="username"
            placeholder="Your username"
            required
            autoFocus
          />
          <button className="btn-primary" type="submit">
            Login
          </button>
          <input type="hidden" name="to" value={to} />
        </div>
      </Form>
      <FormError />
    </div>
  );
}
