import { useReducer } from 'react';
interface History { values: string[]; index: number }
type Action = { type: 'set'; value: string } | { type: 'undo' | 'redo' };
function reduce(state: History, action: Action): History {
  if (action.type === 'undo') return { ...state, index: Math.max(0, state.index - 1) };
  if (action.type === 'redo') return { ...state, index: Math.min(state.values.length - 1, state.index + 1) };
  if (action.type === 'set' && action.value !== state.values[state.index]) { const values = [...state.values.slice(0, state.index + 1), action.value].slice(-40); return { values, index: values.length - 1 }; }
  return state;
}
export function useHistory() {
  const [state, dispatch] = useReducer(reduce, { values: [''], index: 0 });
  return { value: state.values[state.index], set: (value: string) => dispatch({ type: 'set', value }), undo: () => dispatch({ type: 'undo' }), redo: () => dispatch({ type: 'redo' }), canUndo: state.index > 0, canRedo: state.index < state.values.length - 1 };
}
