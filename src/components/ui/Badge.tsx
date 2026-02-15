import * as React from "react";
import { cn } from "../../lib/utils";

export interface BadgeProps extends React.HTMLAttributes<HTMLDivElement> {
    variant?: "default" | "secondary" | "outline" | "success" | "warning" | "error" | "info";
}

function Badge({ className, variant = "default", ...props }: BadgeProps) {
    return (
        <div
            className={cn(
                "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
                {
                    "border-transparent bg-primary text-white shadow hover:bg-primary-hover":
                        variant === "default",
                    "border-transparent bg-dark-secondary text-text-primary hover:bg-dark-secondary/80":
                        variant === "secondary",
                    "text-text-primary border border-dark-border":
                        variant === "outline",
                    "border-transparent bg-success/15 text-success hover:bg-success/25":
                        variant === "success",
                    "border-transparent bg-warning/15 text-warning hover:bg-warning/25":
                        variant === "warning",
                    "border-transparent bg-error/15 text-error hover:bg-error/25":
                        variant === "error",
                    "border-transparent bg-info/15 text-info hover:bg-info/25":
                        variant === "info",
                },
                className
            )}
            {...props}
        />
    );
}

export { Badge };
