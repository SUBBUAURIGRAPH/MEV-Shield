// Common chart configuration with better Y-axis scaling
export const getCommonChartOptions = (title?: string): any => ({
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: {
      position: 'top' as const,
      labels: {
        padding: 15,
        font: {
          size: 12,
          weight: 600,
        },
        usePointStyle: true,
        pointStyle: 'circle',
      },
    },
    title: {
      display: !!title,
      text: title,
      font: {
        size: 16,
        weight: 'bold',
      },
      padding: {
        bottom: 20,
      },
    },
    tooltip: {
      backgroundColor: 'rgba(0, 0, 0, 0.85)',
      titleColor: '#fff',
      bodyColor: '#fff',
      borderColor: '#4CAF50',
      borderWidth: 1,
      padding: 12,
      cornerRadius: 8,
      displayColors: true,
      intersect: false,
      mode: 'index' as const,
      callbacks: {
        label: function(context: any) {
          let label = context.dataset.label || '';
          if (label) {
            label += ': ';
          }
          if (context.parsed.y !== null) {
            label += context.parsed.y.toLocaleString();
          }
          return label;
        },
      },
    },
  },
  scales: {
    x: {
      grid: {
        display: false,
      },
      ticks: {
        font: {
          size: 11,
        },
        color: '#666',
      },
    },
    y: {
      beginAtZero: true,
      grid: {
        color: 'rgba(0, 0, 0, 0.05)',
      },
      ticks: {
        font: {
          size: 11,
        },
        color: '#666',
        padding: 8,
        autoSkip: true,
        maxTicksLimit: 8,
        callback: function(value: any) {
          if (value >= 1000000) {
            return (value / 1000000).toFixed(1) + 'M';
          } else if (value >= 1000) {
            return (value / 1000).toFixed(1) + 'K';
          }
          return value.toLocaleString();
        },
      },
      // Better dynamic scaling based on data
      grace: '5%',
      suggestedMax: undefined,
      adapters: {
        date: {
          locale: 'en-US'
        }
      }
    },
  },
  interaction: {
    intersect: false,
    mode: 'index',
  },
  animation: {
    duration: 750,
    easing: 'easeInOutQuart' as const,
  },
});

// Line chart specific options with improved scaling
export const getLineChartOptions = (title?: string): any => {
  const baseOptions = getCommonChartOptions(title);
  return {
    ...baseOptions,
    plugins: {
      ...baseOptions.plugins,
      tooltip: {
        ...baseOptions.plugins?.tooltip,
        callbacks: {
          ...baseOptions.plugins?.tooltip?.callbacks,
          title: function(tooltipItems: any) {
            return tooltipItems[0].label;
          },
        },
      },
    },
    scales: {
      ...baseOptions.scales,
      y: {
        ...baseOptions.scales.y,
        // Improved scaling for line charts
        grace: '10%',
        beginAtZero: false,
        ticks: {
          ...baseOptions.scales.y.ticks,
          precision: 0,
          maxTicksLimit: 6,
        },
      },
    },
    elements: {
      line: {
        tension: 0.35,
        borderWidth: 2.5,
      },
      point: {
        radius: 4,
        hoverRadius: 6,
        backgroundColor: '#fff',
        borderWidth: 2,
        hoverBorderWidth: 3,
      },
    },
  };
};

// Bar chart specific options with improved scaling
export const getBarChartOptions = (title?: string): any => {
  const baseOptions = getCommonChartOptions(title);
  return {
    ...baseOptions,
    scales: {
      ...baseOptions.scales,
      x: {
        ...baseOptions.scales.x,
        // Better bar spacing
        categoryPercentage: 0.8,
        barPercentage: 0.9,
      },
      y: {
        ...baseOptions.scales.y,
        // Improved Y-axis for bar charts
        grace: '15%',
        ticks: {
          ...baseOptions.scales.y.ticks,
          maxTicksLimit: 5,
          callback: function(value: any) {
            if (title?.includes('ETH') || title?.includes('Value')) {
              return 'Ξ' + value.toFixed(2);
            }
            if (value >= 1000000) {
              return (value / 1000000).toFixed(1) + 'M';
            } else if (value >= 1000) {
              return (value / 1000).toFixed(0) + 'K';
            }
            return value.toLocaleString();
          },
        },
      },
    },
    plugins: {
      ...baseOptions.plugins,
      legend: {
        ...baseOptions.plugins?.legend,
        display: false,
      },
    },
    datasets: {
      bar: {
        borderRadius: 6,
        borderSkipped: false,
        barThickness: 'flex' as any,
        maxBarThickness: 50,
        categoryPercentage: 0.8,
        barPercentage: 0.9,
      },
    },
  };
};

// Doughnut chart options
export const getDoughnutChartOptions = (title?: string): any => ({
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: {
      position: 'right' as const,
      labels: {
        padding: 15,
        font: {
          size: 12,
        },
        generateLabels: function(chart: any) {
          const data = chart.data;
          if (data.labels.length && data.datasets.length) {
            return data.labels.map((label: string, i: number) => {
              const dataset = data.datasets[0];
              const value = dataset.data[i];
              const total = dataset.data.reduce((a: number, b: number) => a + b, 0);
              const percentage = ((value / total) * 100).toFixed(1);
              return {
                text: `${label}: ${percentage}%`,
                fillStyle: dataset.backgroundColor[i],
                hidden: false,
                index: i,
              };
            });
          }
          return [];
        },
      },
    },
    title: {
      display: !!title,
      text: title,
      font: {
        size: 16,
        weight: 'bold',
      },
    },
    tooltip: {
      backgroundColor: 'rgba(0, 0, 0, 0.85)',
      callbacks: {
        label: function(context: any) {
          const label = context.label || '';
          const value = context.parsed;
          const total = context.dataset.data.reduce((a: number, b: number) => a + b, 0);
          const percentage = ((value / total) * 100).toFixed(1);
          return `${label}: ${value.toLocaleString()} (${percentage}%)`;
        },
      },
    },
  },
});

// Generate gradient colors for charts
export const createGradient = (ctx: CanvasRenderingContext2D, color1: string, color2: string) => {
  const gradient = ctx.createLinearGradient(0, 0, 0, 400);
  gradient.addColorStop(0, color1);
  gradient.addColorStop(1, color2);
  return gradient;
};

// Color palette for charts
export const chartColors = {
  primary: '#4CAF50',
  secondary: '#2196F3',
  success: '#4CAF50',
  warning: '#FF9800',
  error: '#F44336',
  info: '#00BCD4',
  purple: '#9C27B0',
  indigo: '#3F51B5',
  pink: '#E91E63',
  teal: '#009688',
  lime: '#CDDC39',
  amber: '#FFC107',
  gradients: {
    green: ['rgba(76, 175, 80, 0.3)', 'rgba(76, 175, 80, 0.05)'],
    blue: ['rgba(33, 150, 243, 0.3)', 'rgba(33, 150, 243, 0.05)'],
    purple: ['rgba(156, 39, 176, 0.3)', 'rgba(156, 39, 176, 0.05)'],
    orange: ['rgba(255, 152, 0, 0.3)', 'rgba(255, 152, 0, 0.05)'],
  },
};