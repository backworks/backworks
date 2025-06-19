import { component$ } from '@builder.io/qwik';

interface MetricCardProps {
  title: string;
  value: string;
  change?: string;
  changeType?: 'positive' | 'negative' | 'neutral';
  icon?: string;
}

export const MetricCard = component$<MetricCardProps>(({ title, value, change, changeType = 'neutral', icon }) => {
  const getChangeColor = () => {
    switch (changeType) {
      case 'positive':
        return 'text-green-600 dark:text-green-400';
      case 'negative':
        return 'text-red-600 dark:text-red-400';
      default:
        return 'text-gray-600 dark:text-gray-400';
    }
  };

  return (
    <div class="rounded-xl bg-white p-6 shadow-sm ring-1 ring-gray-900/5 dark:bg-gray-800 dark:ring-white/10">
      <div class="flex items-center justify-between">
        <div>
          <p class="text-sm font-medium text-gray-600 dark:text-gray-400">{title}</p>
          <p class="text-2xl font-bold text-gray-900 dark:text-white">{value}</p>
          {change && (
            <p class={`text-sm ${getChangeColor()}`}>
              {changeType === 'positive' && '↗'} 
              {changeType === 'negative' && '↘'} 
              {change}
            </p>
          )}
        </div>
        {icon && (
          <div class="rounded-full bg-blue-50 p-3 dark:bg-blue-900/20">
            <div class="h-6 w-6 text-blue-600 dark:text-blue-400" dangerouslySetInnerHTML={icon} />
          </div>
        )}
      </div>
    </div>
  );
});
