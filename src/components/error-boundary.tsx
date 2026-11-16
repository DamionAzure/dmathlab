import { Component, type ErrorInfo, type ReactNode } from "react";

/** Keeps a crash in one view from blanking the whole window. */
export class ErrorBoundary extends Component<
  { children: ReactNode; label?: string },
  { error: Error | null }
> {
  state = { error: null as Error | null };

  static getDerivedStateFromError(error: Error) {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("UI error:", error, info.componentStack);
  }

  render() {
    if (!this.state.error) return this.props.children;
    return (
      <div className="m-4 rounded-lg border border-rose-300 bg-rose-50 p-4 text-sm text-rose-800">
        <strong>{this.props.label ?? "Something went wrong."}</strong>
        <pre className="mt-2 whitespace-pre-wrap text-xs">{this.state.error.message}</pre>
        <button
          className="mt-3 rounded border border-rose-300 bg-white px-3 py-1 text-xs"
          onClick={() => this.setState({ error: null })}
        >
          Try again
        </button>
      </div>
    );
  }
}
