;**********************************************************
; MACROS.ASM
;
; Este programa ilustra el uso de macros para simplificar
; las llamadas al DOS. Cada invocación de una macro se
; substituye por las proposiciones de su definición.
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** MACROS ********************************************

;**********************************************************
; IMPRIME
;
; Esta macro despliega la cadena terminada en '$' dada por
; el parámetro cadena.
;**********************************************************
macro   imprime cadena
        push    ax                      ;; Preserva AX, DX
        push    dx
        mov     dx, offset cadena       ;; Despliega la cadena
        mov     ah, 09h
        int     21h
        pop     dx                      ;; Restaura DX, AX
        pop     ax
endm    imprime

;**********************************************************
; TERMINA
;
; Esta macro termina el programa con el código de salida
; dado por el parámetro codigo.
;**********************************************************
macro   termina codigo
        mov     ah, 04Ch
        mov     al, codigo
        int     21h
endm    termina

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
linea1  db      'Las macros se expanden al ensamblar.', 13, 10, '$'
linea2  db      'Cada uso copia su cuerpo en el codigo.', 13, 10, '$'

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        imprime linea1
        imprime linea2

salir:
        termina 0

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
