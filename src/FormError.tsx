import { useActionData } from "react-router";

export default function FormError() {
  const actionData = useActionData();
  if (!actionData?.error) {
    return null;
  }

  return <p>Error: {actionData.error}</p>;
}
