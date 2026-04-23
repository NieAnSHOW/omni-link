import { useToast as useShadcnToast } from '@/components/ui/toast'

export function useToast() {
  const { toast } = useShadcnToast()

  return {
    success: (message: string) => {
      toast({
        title: '成功',
        description: message,
      })
    },
    error: (message: string) => {
      toast({
        title: '错误',
        description: message,
        variant: 'destructive',
      })
    },
    info: (message: string) => {
      toast({
        title: '提示',
        description: message,
      })
    },
  }
}
