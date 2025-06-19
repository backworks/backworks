import { component$, useSignal, $ } from '@builder.io/qwik';

export interface FormField {
  name: string;
  label: string;
  type: 'text' | 'email' | 'password' | 'number' | 'textarea' | 'select' | 'checkbox' | 'date';
  required?: boolean;
  placeholder?: string;
  options?: string[];
  validation?: string;
  default?: string | number | boolean;
}

export interface WorkflowFormConfig {
  title: string;
  description?: string;
  fields: FormField[];
  endpoint: string;
  method?: 'POST' | 'PUT' | 'PATCH';
  successMessage?: string;
  submitLabel?: string;
}

interface WorkflowFormProps {
  config: WorkflowFormConfig;
  onSubmit?: (data: Record<string, any>) => void;
  onSuccess?: (result: any) => void;
  onError?: (error: any) => void;
}

export const WorkflowForm = component$<WorkflowFormProps>(({ 
  config, 
  onSubmit, 
  onSuccess, 
  onError 
}) => {
  const formData = useSignal<Record<string, any>>({});
  const isSubmitting = useSignal(false);
  const errors = useSignal<Record<string, string>>({});
  const successMessage = useSignal<string>('');

  // Initialize form data with defaults
  config.fields.forEach(field => {
    if (field.default !== undefined) {
      formData.value[field.name] = field.default;
    }
  });

  const validateField = $((field: FormField, value: any): string | null => {
    if (field.required && (!value || value.toString().trim() === '')) {
      return `${field.label} is required`;
    }
    
    if (field.type === 'email' && value) {
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
      if (!emailRegex.test(value)) {
        return 'Please enter a valid email address';
      }
    }
    
    return null;
  });

  const handleSubmit = $(async (event: Event) => {
    event.preventDefault();
    isSubmitting.value = true;
    errors.value = {};
    successMessage.value = '';

    // Validate all fields
    const newErrors: Record<string, string> = {};
    for (const field of config.fields) {
      const error = await validateField(field, formData.value[field.name]);
      if (error) {
        newErrors[field.name] = error;
      }
    }

    if (Object.keys(newErrors).length > 0) {
      errors.value = newErrors;
      isSubmitting.value = false;
      return;
    }

    try {
      // Call custom onSubmit if provided
      if (onSubmit) {
        await onSubmit(formData.value);
      } else {
        // Default API call
        const response = await fetch(config.endpoint, {
          method: config.method || 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(formData.value),
        });

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const result = await response.json();
        
        if (onSuccess) {
          await onSuccess(result);
        }
      }

      successMessage.value = config.successMessage || 'Form submitted successfully!';
      
      // Reset form
      formData.value = {};
      config.fields.forEach(field => {
        if (field.default !== undefined) {
          formData.value[field.name] = field.default;
        }
      });

    } catch (error) {
      console.error('Form submission error:', error);
      if (onError) {
        await onError(error);
      } else {
        errors.value = { _form: error instanceof Error ? error.message : 'Submission failed' };
      }
    } finally {
      isSubmitting.value = false;
    }
  });

  const renderField = (field: FormField) => {
    const baseClasses = "block w-full rounded-md border-0 py-1.5 px-3 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-blue-600 dark:bg-gray-700 dark:text-white dark:ring-gray-600 dark:placeholder:text-gray-400 sm:text-sm sm:leading-6";
    const errorClasses = errors.value[field.name] ? "ring-red-300 focus:ring-red-600" : "";

    switch (field.type) {
      case 'textarea':
        return (
          <textarea
            name={field.name}
            placeholder={field.placeholder}
            required={field.required}
            rows={4}
            class={`${baseClasses} ${errorClasses}`}
            value={formData.value[field.name] || ''}
            onInput$={(e) => {
              formData.value = { ...formData.value, [field.name]: (e.target as HTMLTextAreaElement).value };
            }}
          />
        );

      case 'select':
        return (
          <select
            name={field.name}
            required={field.required}
            class={`${baseClasses} ${errorClasses}`}
            value={formData.value[field.name] || ''}
            onChange$={(e) => {
              formData.value = { ...formData.value, [field.name]: (e.target as HTMLSelectElement).value };
            }}
          >
            <option value="">{`Select ${field.label}`}</option>
            {field.options?.map(option => (
              <option key={option} value={option}>{option}</option>
            ))}
          </select>
        );

      case 'checkbox':
        return (
          <div class="flex items-center">
            <input
              type="checkbox"
              name={field.name}
              class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-600 dark:bg-gray-700 dark:border-gray-600"
              checked={formData.value[field.name] || false}
              onChange$={(e) => {
                formData.value = { ...formData.value, [field.name]: (e.target as HTMLInputElement).checked };
              }}
            />
          </div>
        );

      default:
        return (
          <input
            type={field.type}
            name={field.name}
            placeholder={field.placeholder}
            required={field.required}
            class={`${baseClasses} ${errorClasses}`}
            value={formData.value[field.name] || ''}
            onInput$={(e) => {
              formData.value = { ...formData.value, [field.name]: (e.target as HTMLInputElement).value };
            }}
          />
        );
    }
  };

  return (
    <div class="rounded-lg bg-white p-6 shadow-sm ring-1 ring-gray-900/5 dark:bg-gray-800 dark:ring-white/10">
      <div class="mb-6">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
          {config.title}
        </h3>
        {config.description && (
          <p class="mt-1 text-sm text-gray-600 dark:text-gray-400">
            {config.description}
          </p>
        )}
      </div>

      {successMessage.value && (
        <div class="mb-4 rounded-md bg-green-50 p-4 dark:bg-green-900/20">
          <div class="flex">
            <svg class="h-5 w-5 text-green-400" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
            </svg>
            <div class="ml-3">
              <p class="text-sm font-medium text-green-800 dark:text-green-200">
                {successMessage.value}
              </p>
            </div>
          </div>
        </div>
      )}

      {errors.value._form && (
        <div class="mb-4 rounded-md bg-red-50 p-4 dark:bg-red-900/20">
          <div class="flex">
            <svg class="h-5 w-5 text-red-400" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
            </svg>
            <div class="ml-3">
              <p class="text-sm font-medium text-red-800 dark:text-red-200">
                {errors.value._form}
              </p>
            </div>
          </div>
        </div>
      )}

      <form onSubmit$={handleSubmit}>
        <div class="space-y-4">
          {config.fields.map((field) => (
            <div key={field.name}>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                {field.label}
                {field.required && <span class="text-red-500 ml-1">*</span>}
              </label>
              
              {renderField(field)}
              
              {errors.value[field.name] && (
                <p class="mt-1 text-sm text-red-600 dark:text-red-400">
                  {errors.value[field.name]}
                </p>
              )}
            </div>
          ))}
        </div>

        <div class="mt-6">
          <button
            type="submit"
            disabled={isSubmitting.value}
            class="w-full flex justify-center rounded-md bg-blue-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600 disabled:opacity-50 disabled:cursor-not-allowed dark:bg-blue-500 dark:hover:bg-blue-400"
          >
            {isSubmitting.value ? (
              <>
                <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                Submitting...
              </>
            ) : (
              config.submitLabel || 'Submit'
            )}
          </button>
        </div>
      </form>
    </div>
  );
});
