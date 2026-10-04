import { createContext, useContext, useEffect, useMemo, useSyncExternalStore, type PropsWithChildren } from "react";
import { dispatchAccountAction, loadAccountState } from "./api";
import { AccountController, type AccountState } from "./controller";

type AccountContextValue = AccountState & { controller: AccountController };
const AccountContext = createContext<AccountContextValue | null>(null);

export function AccountProvider({ backendEpoch, children }: PropsWithChildren<{ backendEpoch: string }>) {
  const controller = useMemo(() => new AccountController(backendEpoch, {
    load: loadAccountState,
    dispatch: dispatchAccountAction,
    operationId: () => crypto.randomUUID(),
  }), [backendEpoch]);
  const state = useSyncExternalStore(controller.subscribe, controller.getSnapshot);
  useEffect(() => {
    void controller.start();
    return controller.stop;
  }, [controller]);
  return <AccountContext.Provider value={{ ...state, controller }}>{children}</AccountContext.Provider>;
}

export function useAccount(): AccountContextValue {
  const context = useContext(AccountContext);
  if (!context) throw new Error("AccountProvider is missing");
  return context;
}
