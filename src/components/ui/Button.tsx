import * as React from "react";
import { cn } from "../../lib/utils";
import { Loader2 } from "lucide-react";

export interface ButtonProps
    extends React.ButtonHTMLAttributes<HTMLButtonElement> {
    variant?: "primary" | "secondary" | "outline" | "ghost" | "danger" | "link";
    size?: "default" | "sm" | "lg" | "icon";
    isLoading?: boolean;
    leftIcon?: React.ReactNode;
    rightIcon?: React.ReactNode;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
    (
        {
            className,
            variant = "primary",
            size = "default",
            isLoading,
            leftIcon,
            rightIcon,
            children,
            disabled,
            ...props
        },
        ref
    ) => {
        return (
            <button
                ref={ref}
                disabled={disabled || isLoading}
                className={cn(
                    "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary disabled:pointer-events-none disabled:opacity-50",
                    {
                        // Variants
                        "bg-primary text-white shadow-glow hover:bg-primary-hover":
                            variant === "primary",
                        "bg-dark-surface border border-dark-border text-text-primary hover:bg-dark-surface-hover hover:border-dark-border-hover":
                            variant === "secondary",
                        "border border-dark-border bg-transparent hover:bg-dark-surface-hover text-text-primary":
                            variant === "outline",
                        "hover:bg-dark-surface-hover text-text-secondary hover:text-text-primary":
                            variant === "ghost",
                        "bg-error/10 text-error hover:bg-error/20 border border-error/20":
                            variant === "danger",
                        "text-primary underline-offset-4 hover:underline":
                            variant === "link",

                        // Sizes
                        "h-9 px-4 py-2": size === "default",
                        "h-8 rounded-md px-3 text-xs": size === "sm",
                        "h-10 rounded-md px-8": size === "lg",
                        "h-9 w-9 p-0": size === "icon",
                    },
                    className
                )}
                {...props}
            >
                {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                {!isLoading && leftIcon && <span className="mr-2">{leftIcon}</span>}
                {children}
                {!isLoading && rightIcon && <span className="ml-2">{rightIcon}</span>}
            </button>
        );
    }
);
Button.displayName = "Button";

export { Button };
